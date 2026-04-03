<template>
  <div class="canvas-container" ref="containerRef">
    <!-- Main SVG canvas with hardware acceleration -->
    <svg
      :width="viewportWidth"
      :height="viewportHeight"
      class="canvas"
      :style="{
        transform: canvasTransform.cssTransform.value,
        willChange: 'transform',
        cursor: canvasMode,
      }"
      @wheel.prevent="onMouseWheel"
      @mousemove="onMouseMove"
      @mousedown="onCanvasMouseDown"
    >
      <!-- Render visible nodes -->
      <Node
        v-for="node in visibleNodes"
        :key="node.id"
        :id="node.id"
        :label="node.label"
        :bounds="node.bounds"
        :selected="selectedNodeId === node.id"
        @updatePosition="onNodeUpdate"
        @select="selectedNodeId = $event"
      />
    </svg>

    <!-- Edge layer (rendered on top) -->
    <EdgeConnections
      :edges="edgeData"
      :canvas-width="viewportWidth"
      :canvas-height="viewportHeight"
    />

    <!-- UI Controls -->
    <div class="controls">
      <button @click="canvasTransform.reset()" title="Reset zoom">
        🏠
      </button>
      <button @click="toggleMode" title="Toggle pan/select mode">
        {{ canvasMode === 'grab' ? '✋' : '✌️' }}
      </button>
      <div class="zoom-indicator">
        {{ (canvasTransform.transform.value.scale * 100).toFixed(0) }}%
      </div>
    </div>

    <!-- Status bar -->
    <div class="status-bar">
      <span>Nodes: {{ visibleNodes.length }}</span>
      <span v-if="connectivityState !== 'connected'" class="connection-status">
        {{ connectivityState }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, shallowRef } from 'vue';
import { useCanvasTransform, type Viewport } from '@composables/useCanvasTransform';
import { useYjsCollaboration } from '@composables/useYjsCollaboration';
import { getBridge, type Node as SynapseNode, type Bounds } from '@workers/bridge';
import Node from './Node.vue';
import EdgeConnections from './EdgeConnections.vue';

const props = defineProps<{
  roomId?: string;
  serverUrl?: string;
}>();

const containerRef = ref<HTMLDivElement>();
const viewportWidth = ref(1600);
const viewportHeight = ref(900);

const bridge = getBridge();
const canvasMode = ref<'grab' | 'pointer'>('grab');
const selectedNodeId = ref<number | null>(null);
const visibleNodes = shallowRef<SynapseNode[]>([]);
const edgeData = shallowRef<any[]>([]);
const isPointerDown = ref(false);
const panStart = ref({ x: 0, y: 0 });

// Composables
const canvasTransform = useCanvasTransform(viewportWidth.value, viewportHeight.value);
const collab = useYjsCollaboration(
  props.roomId || 'default-room',
  props.serverUrl
);

const connectivityState = computed(() => collab.connectivityState.value);

/**
 * Initialize everything
 */
onMounted(async () => {
  // Get viewport dimensions
  if (containerRef.value) {
    viewportWidth.value = containerRef.value.clientWidth;
    viewportHeight.value = containerRef.value.clientHeight;
  }

  // Initialize collaboration
  collab.init();

  // Load initial nodes
  const allNodes = collab.getAllNodes();
  for (const node of allNodes) {
    await bridge.insertNode(node.id, node.bounds, node.label);
  }

  // Listen for visibility changes
  const updateVisibleNodes = async () => {
    const nodes = await canvasTransform.getVisibleNodes();
    visibleNodes.value = nodes;

    // Recalculate edge paths
    updateEdgePaths(nodes);
  };

  // Initial update
  await updateVisibleNodes();

  // Listen for collaboration updates
  collab.onAwarenessChange(() => {
    updateVisibleNodes();
  });

  // Listen for CRDT updates
  bridge.on('yjs_update', async () => {
    await updateVisibleNodes();
  });

  // Handle window resize
  const handleResize = () => {
    if (containerRef.value) {
      viewportWidth.value = containerRef.value.clientWidth;
      viewportHeight.value = containerRef.value.clientHeight;
    }
  };

  window.addEventListener('resize', handleResize);
  onUnmounted(() => window.removeEventListener('resize', handleResize));
});

/**
 * Handle mouse wheel for zoom
 */
function onMouseWheel(event: WheelEvent) {
  const zoomDelta = event.deltaY > 0 ? -0.1 : 0.1;
  canvasTransform.zoomToPoint(event.clientX, event.clientY, zoomDelta);
}

/**
 * Handle canvas mouse down for panning
 */
function onCanvasMouseDown(event: MouseEvent) {
  if (canvasMode.value !== 'grab') return;
  if ((event.target as any).tagName !== 'svg') return;

  isPointerDown.value = true;
  panStart.value = { x: event.clientX, y: event.clientY };

  const onMouseMove = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - panStart.value.x;
    const deltaY = moveEvent.clientY - panStart.value.y;

    canvasTransform.pan(deltaX, deltaY);
    panStart.value = { x: moveEvent.clientX, y: moveEvent.clientY };
  };

  const onMouseUp = () => {
    isPointerDown.value = false;
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
  };

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
}

/**
 * Handle mouse move for hit testing
 */
function onMouseMove(event: MouseEvent) {
  if (!isPointerDown.value && canvasMode.value === 'pointer') {
    const worldPos = canvasTransform.screenToWorld(event.clientX, event.clientY);
    // Hit test could be done here for hover effects
  }
}

/**
 * Handle node position update
 */
async function onNodeUpdate(bounds: Bounds) {
  if (selectedNodeId.value !== null) {
    const node = collab.getNode(selectedNodeId.value);
    if (node) {
      const updated = { ...node, bounds };
      collab.upsertNode(updated);
    }
  }
}

/**
 * Update edge paths based on node positions
 */
async function updateEdgePaths(nodes: SynapseNode[]) {
  const edges: any[] = [];

  // This is a simplified example - in real app, edges would come from data
  // Calculate bezier paths between connected nodes
  for (let i = 0; i < nodes.length - 1; i++) {
    const source = nodes[i];
    const target = nodes[i + 1];

    const p0 = {
      x: source.bounds.max.x,
      y: (source.bounds.min.y + source.bounds.max.y) / 2,
    };

    const p3 = {
      x: target.bounds.min.x,
      y: (target.bounds.min.y + target.bounds.max.y) / 2,
    };

    const cp1 = { x: p0.x + 100, y: p0.y };
    const cp2 = { x: p3.x - 100, y: p3.y };

    const pathData = bridge
      ? (await bridge.queryNodes({
          min: { x: Math.min(p0.x, p3.x) - 100, y: Math.min(p0.y, p3.y) - 100 },
          max: { x: Math.max(p0.x, p3.x) + 100, y: Math.max(p0.y, p3.y) + 100 },
        }).then(() =>
          `M${p0.x},${p0.y} C${cp1.x},${cp1.y} ${cp2.x},${cp2.y} ${p3.x},${p3.y}`
        ))
      : `M${p0.x},${p0.y} C${cp1.x},${cp1.y} ${cp2.x},${cp2.y} ${p3.x},${p3.y}`;

    edges.push({
      id: `${source.id}-${target.id}`,
      source: p0,
      target: p3,
      pathData,
    });
  }

  edgeData.value = edges;
}

/**
 * Toggle between pan and select mode
 */
function toggleMode() {
  canvasMode.value = canvasMode.value === 'grab' ? 'pointer' : 'grab';
}
</script>

<style scoped>
.canvas-container {
  width: 100%;
  height: 100%;
  position: relative;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
  overflow: hidden;
  user-select: none;
  font-family: 'Inter', 'Helvetica Neue', sans-serif;
}

.canvas {
  position: absolute;
  top: 0;
  left: 0;
  transform-origin: 0 0;
  background-image:
    radial-gradient(circle, #334155 0.5px, transparent 0.5px);
  background-size: 50px 50px;
  background-position: 0 0;
}

.controls {
  position: absolute;
  bottom: 20px;
  right: 20px;
  display: flex;
  gap: 10px;
  align-items: center;
  background: rgba(30, 41, 59, 0.9);
  padding: 12px 16px;
  border-radius: 8px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(71, 85, 105, 0.5);
  z-index: 100;
}

.controls button {
  background: transparent;
  border: 1px solid #475569;
  color: #f1f5f9;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  transition: all 200ms ease;
}

.controls button:hover {
  background: #334155;
  border-color: #64748b;
}

.zoom-indicator {
  color: #94a3b8;
  font-size: 12px;
  min-width: 50px;
  text-align: right;
}

.status-bar {
  position: absolute;
  bottom: 20px;
  left: 20px;
  color: #94a3b8;
  font-size: 12px;
  display: flex;
  gap: 20px;
  background: rgba(30, 41, 59, 0.9);
  padding: 8px 12px;
  border-radius: 4px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(71, 85, 105, 0.5);
}

.connection-status {
  color: #ef4444;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
