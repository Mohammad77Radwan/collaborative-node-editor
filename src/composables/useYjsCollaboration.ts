/**
 * useYjsCollaboration Composable
 * Manages Yjs CRDT for Real-time Collaborative Editing
 */

import { ref, computed } from 'vue';
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { getBridge, type Node as SynapseNode } from '@workers/bridge';

export function useYjsCollaboration(
  roomName: string,
  serverUrl: string = 'ws://localhost:1234'
) {
  // Local Yjs document
  const yDoc = new Y.Doc();
  const nodesMap = yDoc.getMap('nodes');
  const connectivityState = ref<'connecting' | 'connected' | 'disconnected'>('connecting');
  
  // WebSocket provider for sync
  let provider: WebsocketProvider;

  /**
   * Initialize collaboration
   */
  function init() {
    provider = new WebsocketProvider(serverUrl, roomName, yDoc);

    provider.on('sync', () => {
      connectivityState.value = 'connected';
    });

    provider.on('connection-close', () => {
      connectivityState.value = 'disconnected';
    });

    provider.on('connection-error', (error) => {
      console.error('WebSocket error:', error);
      connectivityState.value = 'disconnected';
    });

    // Observe map changes for differential syncing
    nodesMap.observeDeep((events) => {
      const bridge = getBridge();
      const diff: Record<number, SynapseNode | null> = {};

      for (const event of events) {
        if (event.path && event.path.length > 0) {
          const nodeId = parseInt(event.path[0] as string, 10);
          const node = nodesMap.get(event.path[0] as string);

          if (node === undefined) {
            diff[nodeId] = null; // Deletion
          } else {
            diff[nodeId] = node;
          }
        }
      }

      // Send batch update to worker
      if (Object.keys(diff).length > 0) {
        bridge.batchUpdate(diff);
      }
    });
  }

  /**
   * Insert or update a node
   */
  function upsertNode(node: SynapseNode) {
    const yNode = {
      id: node.id,
      label: node.label,
      bounds: node.bounds,
      updatedAt: Date.now(),
      updatedBy: provider?.awareness?.clientID,
    };
    nodesMap.set(node.id.toString(), yNode);
  }

  /**
   * Remove a node
   */
  function removeNode(nodeId: number) {
    nodesMap.delete(nodeId.toString());
  }

  /**
   * Get node from collaborative state
   */
  function getNode(nodeId: number): SynapseNode | null {
    const node = nodesMap.get(nodeId.toString());
    return node || null;
  }

  /**
   * Get all nodes
   */
  function getAllNodes(): SynapseNode[] {
    const nodes: SynapseNode[] = [];
    nodesMap.forEach((node) => {
      nodes.push(node);
    });
    return nodes;
  }

  /**
   * Set awareness metadata (cursor, selection, etc)
   */
  function setAwareness(data: Record<string, any>) {
    if (provider?.awareness) {
      provider.awareness.setLocalState(data);
    }
  }

  /**
   * Subscribe to awareness changes
   */
  function onAwarenessChange(callback: (changes: Map<number, any>) => void) {
    if (provider?.awareness) {
      provider.awareness.on('change', (changes) => {
        callback(changes);
      });
    }
  }

  /**
   * Get all awareness states
   */
  function getAwarenessStates(): Map<number, any> {
    if (provider?.awareness) {
      return provider.awareness.getStates();
    }
    return new Map();
  }

  /**
   * Undo (if yjs-undo is integrated)
   */
  function undo() {
    // This would require y-undo plugin
  }

  /**
   * Redo
   */
  function redo() {
    // This would require y-undo plugin
  }

  /**
   * Cleanup
   */
  function destroy() {
    if (provider) {
      provider.destroy();
    }
    yDoc.destroy();
  }

  return {
    yDoc,
    nodesMap,
    connectivityState: computed(() => connectivityState.value),
    init,
    upsertNode,
    removeNode,
    getNode,
    getAllNodes,
    setAwareness,
    onAwarenessChange,
    getAwarenessStates,
    undo,
    redo,
    destroy,
  };
}
