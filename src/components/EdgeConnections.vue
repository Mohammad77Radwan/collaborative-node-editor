<template>
  <svg
    :width="canvasWidth"
    :height="canvasHeight"
    class="edges-container"
    style="will-change: transform; position: absolute; top: 0; left: 0"
  >
    <!-- Single bundled path for all edges -->
    <path
      :d="bundledPathData"
      class="edge-path"
      style="
        fill: none;
        stroke: #64748b;
        stroke-width: 2;
        stroke-linecap: round;
        stroke-linejoin: round;
      "
    />
    <!-- Individual edge highlights on hover -->
    <path
      v-for="edge in visibleEdges"
      :key="edge.id"
      :d="edge.pathData"
      class="edge-highlight"
      :class="{ active: activeEdgeId === edge.id }"
      @mouseenter="activeEdgeId = edge.id"
      @mouseleave="activeEdgeId = null"
      style="
        fill: none;
        stroke: transparent;
        stroke-width: 10;
        cursor: pointer;
        transition: stroke 200ms ease;
      "
    />
  </svg>
</template>

<script setup lang="ts">
import { ref, computed, shallowRef, watch, onUnmounted } from 'vue';
import { getBridge } from '@workers/bridge';

export interface EdgeData {
  id: number;
  source: { x: number; y: number };
  target: { x: number; y: number };
  pathData: string;
}

const props = defineProps<{
  edges: EdgeData[];
  canvasWidth: number;
  canvasHeight: number;
}>();

const emit = defineEmits<{
  edgeClick: [edgeId: number];
  edgeHover: [edgeId: number | null];
}>();

const activeEdgeId = ref<number | null>(null);
const bridge = getBridge();

// Compute visible edges (those that intersect viewport)
const visibleEdges = shallowRef<EdgeData[]>(props.edges);

// Bundle all edges into single SVG path data
const bundledPathData = computed(() => {
  const paths: string[] = [];
  
  for (const edge of visibleEdges.value) {
    paths.push(edge.pathData.substring(1)); // Remove 'M' from start
  }
  
  // Start with first move command, then concatenate all paths
  if (paths.length === 0) return '';
  return 'M' + paths.join(' M');
});

// Watch for edge updates
watch(
  () => props.edges,
  (newEdges) => {
    visibleEdges.value = newEdges;
  },
  { deep: false }
);

// Listen for Yjs updates
const unsubscribe = bridge.on('yjs_update', (msg) => {
  // Edge paths may need recalculation on node updates
  emit('edgeHover', activeEdgeId.value);
});

// Cleanup
onUnmounted(() => {
  unsubscribe?.();
});
</script>

<style scoped>
.edges-container {
  pointer-events: none;
  user-select: none;
}

.edge-highlight:hover {
  stroke: #3b82f6 !important;
  stroke-width: 12 !important;
}

.edge-highlight.active {
  stroke: #1d4ed8 !important;
  stroke-width: 14 !important;
  z-index: 10;
}
</style>
