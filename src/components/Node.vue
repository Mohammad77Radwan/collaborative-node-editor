<template>
  <g
    :transform="`translate(${bounds.min.x},${bounds.min.y})`"
    class="node-group"
    @mousedown="onMouseDown"
    @contextmenu.prevent="onContextMenu"
  >
    <!-- Node background with rounded corners -->
    <rect
      :width="width"
      :height="height"
      :class="{ selected, dragging }"
      class="node-bg"
      rx="8"
      ry="8"
      :style="{
        fill: selected ? '#3b82f6' : '#1e293b',
        stroke: selected ? '#1d4ed8' : '#475569',
        strokeWidth: selected ? 2 : 1,
        cursor: 'grab',
        filter: selected ? 'drop-shadow(0 0 8px rgba(59, 130, 246, 0.5))' : 'none',
      }"
    />

    <!-- Input ports (left side) -->
    <g v-for="(port, i) in inputPorts" :key="`in-${i}`" class="port input-port">
      <circle
        :cx="0"
        :cy="height * ((i + 1) / (inputPorts.length + 1))"
        r="6"
        :style="{
          fill: '#64748b',
          cursor: 'pointer',
          transition: 'fill 200ms ease',
        }"
        @mouseenter="$event.target.setAttribute('fill', '#3b82f6')"
        @mouseleave="$event.target.setAttribute('fill', '#64748b')"
        @click="onPortClick('input', i)"
      />
    </g>

    <!-- Output ports (right side) -->
    <g v-for="(port, i) in outputPorts" :key="`out-${i}`" class="port output-port">
      <circle
        :cx="width"
        :cy="height * ((i + 1) / (outputPorts.length + 1))"
        r="6"
        :style="{
          fill: '#64748b',
          cursor: 'pointer',
          transition: 'fill 200ms ease',
        }"
        @mouseenter="$event.target.setAttribute('fill', '#10b981')"
        @mouseleave="$event.target.setAttribute('fill', '#64748b')"
        @click="onPortClick('output', i)"
      />
    </g>

    <!-- Label text -->
    <text
      :x="width / 2"
      :y="height / 2 + 4"
      text-anchor="middle"
      dominant-baseline="middle"
      class="node-label"
      :style="{
        fill: '#f1f5f9',
        fontSize: '14px',
        fontWeight: '500',
        pointerEvents: 'none',
      }"
    >
      {{ label }}
    </text>
  </g>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { getBridge, type Bounds } from '@workers/bridge';

export interface Port {
  id: number;
  label: string;
  type: string;
}

const props = defineProps<{
  id: number;
  label: string;
  bounds: Bounds;
  inputPorts?: Port[];
  outputPorts?: Port[];
  selected?: boolean;
}>();

const emit = defineEmits<{
  updatePosition: [bounds: Bounds];
  portConnect: [type: 'input' | 'output', portIndex: number];
  select: [id: number];
}>();

const bridge = getBridge();
const dragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const currentBounds = ref(props.bounds);

const width = computed(() => props.bounds.max.x - props.bounds.min.x);
const height = computed(() => props.bounds.max.y - props.bounds.min.y);
const inputPorts = computed(() => props.inputPorts || []);
const outputPorts = computed(() => props.outputPorts || []);

/**
 * Handle mouse down for dragging
 */
function onMouseDown(event: MouseEvent) {
  if ((event.target as any).tagName === 'circle') return;
  
  emit('select', props.id);
  dragging.value = true;
  dragStart.value = { x: event.clientX, y: event.clientY };

  const onMouseMove = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - dragStart.value.x;
    const deltaY = moveEvent.clientY - dragStart.value.y;

    const newBounds: Bounds = {
      min: {
        x: currentBounds.value.min.x + deltaX,
        y: currentBounds.value.min.y + deltaY,
      },
      max: {
        x: currentBounds.value.max.x + deltaX,
        y: currentBounds.value.max.y + deltaY,
      },
    };

    currentBounds.value = newBounds;
    emit('updatePosition', newBounds);

    dragStart.value = { x: moveEvent.clientX, y: moveEvent.clientY };
  };

  const onMouseUp = () => {
    dragging.value = false;
    // Sync with server via Yjs/WebWorker
    bridge.updateNode(props.id, currentBounds.value);

    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
  };

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
}

/**
 * Handle context menu for node options
 */
function onContextMenu(event: MouseEvent) {
  // Could trigger context menu for delete, duplicate, etc
  console.log('Context menu on node', props.id);
}

/**
 * Handle port click for connections
 */
function onPortClick(type: 'input' | 'output', portIndex: number) {
  emit('portConnect', type, portIndex);
}
</script>

<style scoped>
.node-group {
  user-select: none;
}

.node-bg {
  transition: filter 200ms ease;
}

.node-bg.selected {
  filter: drop-shadow(0 0 12px rgba(59, 130, 246, 0.7));
}

.node-label {
  font-family: 'Inter', 'Helvetica Neue', sans-serif;
  text-rendering: geometricPrecision;
}

.port {
  transition: opacity 200ms ease;
}

.port:hover {
  opacity: 1;
}
</style>
