/**
 * Base class for all shapes.
 * A shape in feakin is a separate class that is used to draw a shape in SVG, Canvas.
 *
 * ## Custom Shapes
 * To extend from this class, the basic code looks as follows.
 *
 * ```javascript
 *
 * ```
 *
 */

export class Shape {
  x: number;
  y: number;
  width: number;
  height: number;

  constructor(x: number = 0, y: number = 0, width: number = 0, height: number = 0) {
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
  }

  /**
   * @param scale The scale to be applied to the shape.
   */
  scale(scale: number) {
    this.width *= scale;
    this.height *= scale;
  }

  point() {
  }

  rotate() {
  }
}
