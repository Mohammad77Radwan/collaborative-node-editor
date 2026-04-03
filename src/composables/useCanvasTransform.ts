/**
 * useCanvasTransform Composable
 * Manages infinite canvas transformations with stationary-point zoom
 * Integrates with Quadtree for viewport-based culling
 */

import { ref, computed, shallowRef } from 'vue';
import { getBridge, type Bounds } from '@workers/bridge';

export interface Transform {
  x: number;
  y: number;
  scale: number;
}

export interface Viewport {
  bounds: Bounds;
  centerX: number;
  centerY: number;
  scale: number;
}

export function useCanvasTransform(canvasWidth: number, canvasHeight: number) {
  // Use shallowRef to bypass Vue's deep reactivity overhead
  const transform = shallowRef<Transform>({ x: 0, y: 0, scale: 1 });
  
  // Viewport for spatial queries
  const viewport = shallowRef<Viewport>({
    bounds: { min: { x: 0, y: 0 }, max: { x: canvasWidth, y: canvasHeight } },
    centerX: canvasWidth / 2,
    centerY: canvasHeight / 2,
    scale: 1,
  });

  // CSS transform string for hardware acceleration
  const cssTransform = computed(() => {
    const { x, y, scale } = transform.value;
    // Use translate3d to stay on GPU compositor
    return `translate3d(${x}px, ${y}px, 0) scale(${scale})`;
  });

  /**
   * Stationary-point zoom algorithm
   * Ensures the point under the cursor remains stationary during zoom
   */
  function zoomToPoint(mouseX: number, mouseY: number, zoomDelta: number) {
    const { x: currentX, y: currentY, scale: currentScale } = transform.value;

    // Calculate world position of mouse cursor
    const worldX = (mouseX - currentX) / currentScale;
    const worldY = (mouseY - currentY) / currentScale;

    // Apply zoom
    const newScale = Math.max(0.1, Math.min(currentScale * (1 + zoomDelta), 20));
    const scaleFactor = newScale / currentScale;

    // Calculate new translation to keep world point stationary
    const newX = mouseX - worldX * newScale;
    const newY = mouseY - worldY * newScale;

    transform.value = { x: newX, y: newY, scale: newScale };
    updateViewport();
  }

  /**
   * Pan the canvas
   */
  function pan(deltaX: number, deltaY: number) {
    const { x, y, scale } = transform.value;
    transform.value = {
      x: x + deltaX,
      y: y + deltaY,
      scale,
    };
    updateViewport();
  }

  /**
   * Update viewport bounds for spatial queries
   * This is used to cull nodes outside the visible area
   */
  function updateViewport() {
    const { x, y, scale } = transform.value;

    // Calculate world-space bounds
    const minX = -x / scale;
    const minY = -y / scale;
    const maxX = (canvasWidth - x) / scale;
    const maxY = (canvasHeight - y) / scale;

    viewport.value = {
      bounds: {
        min: { x: minX, y: minY },
        max: { x: maxX, y: maxY },
      },
      centerX: (minX + maxX) / 2,
      centerY: (minY + maxY) / 2,
      scale,
    };
  }

  /**
   * Get visible nodes from Quadtree
   */
  async function getVisibleNodes() {
    const bridge = getBridge();
    return await bridge.queryNodes(viewport.value.bounds);
  }

  /**
   * Convert screen coordinates to world coordinates
   */
  function screenToWorld(screenX: number, screenY: number) {
    const { x, y, scale } = transform.value;
    return {
      x: (screenX - x) / scale,
      y: (screenY - y) / scale,
    };
  }

  /**
   * Convert world coordinates to screen coordinates
   */
  function worldToScreen(worldX: number, worldY: number) {
    const { x, y, scale } = transform.value;
    return {
      x: worldX * scale + x,
      y: worldY * scale + y,
    };
  }

  /**
   * Reset transform to identity
   */
  function reset() {
    transform.value = { x: 0, y: 0, scale: 1 };
    updateViewport();
  }

  /**
   * Fit all nodes in viewport (if bounds provided)
   */
  function fitBounds(bounds: Bounds, padding: number = 50) {
    const width = bounds.max.x - bounds.min.x;
    const height = bounds.max.y - bounds.min.y;

    const scaleX = (canvasWidth - padding * 2) / width;
    const scaleY = (canvasHeight - padding * 2) / height;
    const scale = Math.min(scaleX, scaleY);

    const centerX = (bounds.min.x + bounds.max.x) / 2;
    const centerY = (bounds.min.y + bounds.max.y) / 2;

    transform.value = {
      x: canvasWidth / 2 - centerX * scale,
      y: canvasHeight / 2 - centerY * scale,
      scale,
    };
    updateViewport();
  }

  // Initialize viewport
  updateViewport();

  return {
    transform: computed(() => transform.value),
    viewport: computed(() => viewport.value),
    cssTransform,
    zoomToPoint,
    pan,
    getVisibleNodes,
    screenToWorld,
    worldToScreen,
    reset,
    fitBounds,
  };
}
