/**
 * WebWorker Bridge for Synapse-Ultra
 * Main thread API for communicating with the WASM worker
 */

export interface Bounds {
  min: { x: number; y: number };
  max: { x: number; y: number };
}

export interface Node {
  id: number;
  label: string;
  bounds: Bounds;
}

type MessageCallback = (data: any) => void;

export class SynapseWorkerBridge {
  private worker: Worker;
  private requestIdCounter = 0;
  private pendingRequests = new Map<number, MessageCallback>();
  private listeners = new Map<string, MessageCallback[]>();

  constructor() {
    // Import worker as module
    this.worker = new Worker(
      new URL('./synapse.worker.ts', import.meta.url),
      { type: 'module' }
    );

    // Setup message handler
    this.worker.onmessage = this.handleWorkerMessage.bind(this);
    this.worker.onerror = this.handleWorkerError.bind(this);

    // Initialize worker
    this.worker.postMessage({ type: 'init' });
  }

  /**
   * Handle messages from worker
   */
  private handleWorkerMessage(event: MessageEvent) {
    const msg = event.data;

    // Handle request responses
    if (msg.requestId !== undefined) {
      const callback = this.pendingRequests.get(msg.requestId);
      if (callback) {
        callback(msg);
        this.pendingRequests.delete(msg.requestId);
      }
      return;
    }

    // Handle event broadcasts
    if (msg.type) {
      const callbacks = this.listeners.get(msg.type) || [];
      for (const callback of callbacks) {
        callback(msg);
      }
    }
  }

  /**
   * Error handler
   */
  private handleWorkerError(error: ErrorEvent) {
    console.error('Worker error:', error.message);
    this.emit('error', { message: error.message });
  }

  /**
   * Send message to worker and get response
   */
  private async request(message: any): Promise<any> {
    return new Promise((resolve) => {
      const requestId = ++this.requestIdCounter;
      message.requestId = requestId;

      this.pendingRequests.set(requestId, (response) => {
        resolve(response);
      });

      this.worker.postMessage(message);
    });
  }

  /**
   * Insert a node
   */
  async insertNode(id: number, bounds: Bounds, label: string): Promise<void> {
    this.worker.postMessage({
      type: 'node_insert',
      id,
      bounds,
      label,
    });
  }

  /**
   * Update a node's bounds
   */
  async updateNode(id: number, bounds: Bounds): Promise<void> {
    this.worker.postMessage({
      type: 'node_update',
      id,
      bounds,
    });
  }

  /**
   * Remove a node
   */
  async removeNode(id: number): Promise<void> {
    this.worker.postMessage({
      type: 'node_remove',
      id,
    });
  }

  /**
   * Query nodes in a region - returns list of nodes
   */
  async queryNodes(bounds: Bounds): Promise<Node[]> {
    const response = await this.request({
      type: 'query',
      bounds,
    });
    return response.nodes || [];
  }

  /**
   * Hit test at point
   */
  async hitTest(x: number, y: number): Promise<number | null> {
    const response = await this.request({
      type: 'hit_test',
      x,
      y,
    });
    return response.nodeId;
  }

  /**
   * Hit test with radius
   */
  async hitTestRadius(x: number, y: number, radius: number = 10): Promise<number[]> {
    const response = await this.request({
      type: 'hit_test_radius',
      x,
      y,
      radius,
    });
    return response.nodeIds || [];
  }

  /**
   * Batch update nodes (optimized for CRDT diffs)
   */
  async batchUpdate(diff: Record<number, Node | null>): Promise<void> {
    this.worker.postMessage({
      type: 'batch_update',
      diff,
    });
  }

  /**
   * Apply Yjs update
   */
  applyYjsUpdate(update: Uint8Array): void {
    this.worker.postMessage({
      type: 'yjs_update',
      data: update,
    });
  }

  /**
   * Listen for events
   */
  on(eventType: string, callback: MessageCallback): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, []);
    }
    this.listeners.get(eventType)!.push(callback);

    // Return unsubscribe function
    return () => {
      const callbacks = this.listeners.get(eventType) || [];
      const idx = callbacks.indexOf(callback);
      if (idx > -1) {
        callbacks.splice(idx, 1);
      }
    };
  }

  /**
   * Emit event
   */
  private emit(eventType: string, data: any): void {
    const callbacks = this.listeners.get(eventType) || [];
    for (const callback of callbacks) {
      callback(data);
    }
  }

  /**
   * Terminate worker
   */
  terminate(): void {
    this.worker.terminate();
  }
}

// Global instance
let bridgeInstance: SynapseWorkerBridge | null = null;

export function initBridge(): SynapseWorkerBridge {
  if (!bridgeInstance) {
    bridgeInstance = new SynapseWorkerBridge();
  }
  return bridgeInstance;
}

export function getBridge(): SynapseWorkerBridge {
  if (!bridgeInstance) {
    throw new Error('Bridge not initialized. Call initBridge() first.');
  }
  return bridgeInstance;
}
