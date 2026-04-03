/**
 * Core WebWorker for Synapse-Ultra
 * Handles WASM computation, Yjs sync, and differential updates
 */

import * as Y from 'yjs';
import type { Awareness } from 'y-protocols/awareness';

interface NodeUpdateMessage {
  type: 'node_update' | 'node_remove' | 'node_insert' | 'batch_update';
  id?: number;
  bounds?: { min: { x: number; y: number }; max: { x: number; y: number } };
  label?: string;
  timestamp?: number;
  diff?: Record<number, any>;
}

interface QueryMessage {
  type: 'query';
  requestId: number;
  bounds: { min: { x: number; y: number }; max: { x: number; y: number } };
}

interface HitTestMessage {
  type: 'hit_test' | 'hit_test_radius';
  requestId: number;
  x: number;
  y: number;
  radius?: number;
}

type WorkerMessage = NodeUpdateMessage | QueryMessage | HitTestMessage;

// State management
let wasmReady = false;
let wasmModule: any = null;
let yDocument: Y.Doc;
let nodesMap: Y.Map<any>;
let lastSyncState: Map<number, any> = new Map();

/**
 * Initialize WASM module dynamically
 */
async function initWasm() {
  try {
    // Dynamic import of WASM module
    wasmModule = await import('synapse-core');
    wasmModule.init_quadtree(0, 0, 10000, 10000);
    wasmReady = true;
    self.postMessage({ type: 'wasm_ready' });
  } catch (err) {
    console.error('WASM init failed:', err);
    self.postMessage({ type: 'wasm_error', error: String(err) });
  }
}

/**
 * Initialize Yjs for collaborative state
 */
function initYjs() {
  yDocument = new Y.Doc();
  nodesMap = yDocument.getMap('nodes');

  // Observe changes to sync back to main thread
  nodesMap.observe((event) => {
    const delta = generateDelta(event.changes);
    self.postMessage({
      type: 'yjs_update',
      delta,
      timestamp: Date.now(),
    });
  });
}

/**
 * Generate differential update (delta)
 */
function generateDelta(changes: Map<string, any>): Record<string, any> {
  const delta: Record<string, any> = {};
  
  changes.forEach((change, key) => {
    const nodeId = parseInt(key, 10);
    const newState = nodesMap.get(key);
    const oldState = lastSyncState.get(nodeId);

    // Only include changed fields
    if (JSON.stringify(newState) !== JSON.stringify(oldState)) {
      delta[key] = newState;
      lastSyncState.set(nodeId, JSON.parse(JSON.stringify(newState)));
    }
  });

  return delta;
}

/**
 * Insert or update node in WASM quadtree
 */
function upsertNode(id: number, bounds: any, label: string) {
  if (!wasmReady || !wasmModule) return;

  try {
    wasmModule.insert_node(
      id,
      bounds.min.x,
      bounds.min.y,
      bounds.max.x,
      bounds.max.y,
      label
    );

    // Also update Yjs for collaborative sync
    nodesMap.set(id.toString(), {
      id,
      bounds,
      label,
      updatedAt: Date.now(),
    });
  } catch (err) {
    console.error('Failed to insert node:', err);
  }
}

/**
 * Remove node from both WASM and Yjs
 */
function removeNode(id: number) {
  if (!wasmReady || !wasmModule) return;

  try {
    wasmModule.remove_node(id);
    nodesMap.delete(id.toString());
  } catch (err) {
    console.error('Failed to remove node:', err);
  }
}

/**
 * Query nodes in region
 */
function queryNodes(requestId: number, bounds: any) {
  if (!wasmReady || !wasmModule) {
    self.postMessage({ requestId, error: 'WASM not ready' });
    return;
  }

  try {
    const result = wasmModule.query_nodes(
      bounds.min.x,
      bounds.min.y,
      bounds.max.x,
      bounds.max.y
    );
    const nodes = JSON.parse(result);
    
    self.postMessage({
      type: 'query_result',
      requestId,
      nodes,
      timestamp: Date.now(),
    });
  } catch (err) {
    console.error('Query failed:', err);
    self.postMessage({ requestId, error: String(err) });
  }
}

/**
 * Hit test at point
 */
function hitTest(requestId: number, x: number, y: number) {
  if (!wasmReady || !wasmModule) {
    self.postMessage({ requestId, error: 'WASM not ready' });
    return;
  }

  try {
    const nodeId = wasmModule.hit_test(x, y);
    
    self.postMessage({
      type: 'hit_test_result',
      requestId,
      nodeId: nodeId === 0 ? null : nodeId,
      timestamp: Date.now(),
    });
  } catch (err) {
    console.error('Hit test failed:', err);
    self.postMessage({ requestId, error: String(err) });
  }
}

/**
 * Hit test with radius
 */
function hitTestRadius(requestId: number, x: number, y: number, radius: number) {
  if (!wasmReady || !wasmModule) {
    self.postMessage({ requestId, error: 'WASM not ready' });
    return;
  }

  try {
    const result = wasmModule.hit_test_radius(x, y, radius);
    const nodeIds = JSON.parse(result);
    
    self.postMessage({
      type: 'hit_test_result',
      requestId,
      nodeIds,
      timestamp: Date.now(),
    });
  } catch (err) {
    console.error('Hit test radius failed:', err);
    self.postMessage({ requestId, error: String(err) });
  }
}

/**
 * Process Yjs WebSocket updates
 */
function processYjsUpdate(update: Uint8Array) {
  try {
    Y.applyUpdate(yDocument, update);
  } catch (err) {
    console.error('Failed to apply Yjs update:', err);
  }
}

/**
 * Main message handler
 */
self.onmessage = (event) => {
  const msg: WorkerMessage = event.data;

  switch (msg.type) {
    case 'init':
      initWasm();
      initYjs();
      break;

    case 'node_insert':
      if (msg.id !== undefined && msg.bounds && msg.label) {
        upsertNode(msg.id, msg.bounds, msg.label);
      }
      break;

    case 'node_update':
      if (msg.id !== undefined && msg.bounds && wasmReady && wasmModule) {
        wasmModule.update_node(
          msg.id,
          msg.bounds.min.x,
          msg.bounds.min.y,
          msg.bounds.max.x,
          msg.bounds.max.y
        );
      }
      break;

    case 'node_remove':
      if (msg.id !== undefined) {
        removeNode(msg.id);
      }
      break;

    case 'batch_update':
      if (msg.diff) {
        Object.entries(msg.diff).forEach(([id, node]: [string, any]) => {
          if (node === null) {
            removeNode(parseInt(id, 10));
          } else {
            upsertNode(parseInt(id, 10), node.bounds, node.label);
          }
        });
      }
      break;

    case 'query':
      queryNodes((event as any).requestId, msg.bounds);
      break;

    case 'hit_test':
      hitTest((event as any).requestId, msg.x, msg.y);
      break;

    case 'hit_test_radius':
      hitTestRadius((event as any).requestId, msg.x, msg.y, msg.radius || 10);
      break;

    case 'yjs_update':
      processYjsUpdate((event as any).data);
      break;
  }
};

// Signal readiness
self.postMessage({ type: 'worker_ready' });
