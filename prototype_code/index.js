function getRandomInt(max) {
    return Math.floor(Math.random() * (max + 1));
}

class Vec2 {
    x;
    y;
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    static fromScreenXY(x, y) {
        return new Vec2(Vec2.fromScreenX(x), Vec2.fromScreenY(y));

    }
    static fromScreenX(x) {
        return (x - BOX_SCREEN_ORIGIN.x) / ROOT_BOX_SIZE;
    }
    static fromScreenY(y) {

        return (canvas.height - y - BOX_SCREEN_ORIGIN.x) / ROOT_BOX_SIZE;
    }
    screenX() {
        return ROOT_BOX_SIZE * this.x + BOX_SCREEN_ORIGIN.x;
    }
    screenY() {
        return canvas.height - (this.y * ROOT_BOX_SIZE + BOX_SCREEN_ORIGIN.y);
    }

    minus(rhs) {
        return new Vec2(
            this.x - rhs.x,
            this.y - rhs.y
        )
    }
    plus(rhs) {
        return new Vec2(
            this.x + rhs.x,
            this.y + rhs.y
        )
    }
    times(scalar) {
        return new Vec2(
            scalar * this.x,
            scalar * this.y
        )
    }
    divide(scalar) {
        return new Vec2(
            this.x / scalar,
            this.y / scalar
        )
    }
    magnitude() {
        return Math.sqrt(this.x * this.x + this.y * this.y)
    }
    clone() {
        return new Vec2(this.x, this.y)
    }
}
class Ray {
    origin;
    direction;
    constructor(origin, direction) {
        this.origin = origin;
        this.direction = direction
    }
    at(t) {
        const origin = this.origin.plus(this.direction.times(t));
        return new Ray(origin, this.direction)
    }
    normalize() {
        const direction = this.direction.divide(this.direction.magnitude());
        return new Ray(this.origin, direction)
    }
}
class Node {
    size;
    children;
    is_solid;
    constructor(size) {
        this.size = size;
        this.children = null;
        if (this.size >= 1) {
            const random_number = getRandomInt(1);
            console.log(random_number);
            if (random_number == 1) {
                console.log("creating children for size: %d", this.size);
                const new_size = size - 1;
                this.children = [new Node(new_size), new Node(new_size), new Node(new_size), new Node(new_size)]
            } else {
                const random_number = getRandomInt(1);
                if (random_number == 1) {
                    this.is_solid = true;
                } else {
                    this.is_solid = false;
                }
            }
        }
        if (this.children != null) {
            if (this.children.length != 4) {
                throw "Invalid children size"
            }
        }

        this.size = size;
    }
    getColor() {
        if (this.children != null) {
            const colors = ["#25008bff", "#ff00b3ff", "#009908ff", "#ff0000ff", "#3700ffff"];
            const colors_index = Math.min(this.size - 1, colors.length - 1);
            return colors[colors_index];
        } else {
            if (this.is_solid) {
                return "#ffffff"
            } else {
                return "#ffffff00"
            }

        }


    }
    draw(ctx, origin = new Vec2(0., 0.)) {


        ctx.fillStyle = this.getColor();
        let lower_left = origin;
        let upper_right = new Vec2(Math.pow(2, this.size), Math.pow(2, this.size)).plus(origin);
        let width = upper_right.screenX() - lower_left.screenX();
        let height = upper_right.screenY() - lower_left.screenY();

        ctx.fillRect(lower_left.screenX(), lower_left.screenY(), width, height);


        if (this.children != null) {

            let world_children = this.childrenXYWorld();



            for (let i = 0; i < 4; i++) {

                const children_origin = world_children[i].plus(origin);

                this.children[i].draw(ctx, children_origin);


            }


        }
        ctx.strokeStyle = "#1b0707ff"
        ctx.strokeRect(lower_left.screenX(), lower_left.screenY(), width, height);
    }
    getSideLength() {
        return Math.pow(2, this.size)
    }
    isRayOutside(ray) {
        return ray.origin.x <= 0. || ray.origin.x >= Math.pow(2., this.size) || ray.origin.y <= 0. || ray.origin.y >= Math.pow(2., this.size)
    }
    getNode(x, y) {
        const side_length = Math.pow(2, this.size);
        if (x >= side_length) {
            throw `X is to large, x: ${x}`
        }

        if (this.children == null) {
            return this
        } else {
            const side_length_div2 = Math.pow(2, this.size - 1)
            const x_get = Math.floor(x / side_length_div2);
            const y_get = Math.floor(y / side_length_div2);
            const offset_x = x - x_get * side_length_div2;
            const offset_y = y - y_get * side_length_div2;
            let get_node = this.children[Node.getChildIndex(x_get, y_get)]
            return get_node.getNode(offset_x, offset_y)
            console.error(`get node not implemented yet for children, returning this, x: ${x}, y: ${y}, x_get: ${x_get}, y_get: ${y_get}, offset: (${offset_x},${offset_y})`); return this;
        }

    }
    isBlockPosInRange(x, y) {
        if (x < 0) {
            return false;
        }
        if (y < 0) {
            return false;
        }
        if (x >= Math.pow(2, this.size)) {
            return false
        }
        if (y >= Math.pow(2, this.size)) {
            return false
        }
        return true;
    }
    getStepSize(x, y) {
        //gets the min step size

        const node = this.getNode(x, y);
        if (node.children == null) {
            return Math.pow(2, node.size)
        } else {
            throw "Children is NULL!!!"

        }

    }
    // returns a floored to the nearest multiple of b
    _floor_value(a, b) {
        return Math.floor(a) - Math.floor(a) % b
    }
    _rayIteration(block_x, block_y, position, direction, step_size) {
        if (direction.x == 0 || direction.y == 0) {
            return [[], {}]
        }

        let block_history = [];
        let debug_text = "";
        let output = []
        let normal = new Vec2(0., 0.)
        for (let i = 0; i < 20; i++) {
            block_history.push(new Vec2(block_x, block_y));
            if (this.isBlockPosInRange(block_x, block_y) === false) {
                debug_text += `<br>breaking at block pos <${block_x}, ${block_y}>`
                break;
            }

            step_size = this.getStepSize(block_x, block_y);


            debug_text += `<br>block: <${block_x}, ${block_y}>`
            debug_text += `<br> position: <${position.x.toFixed(2)}, ${position.y.toFixed(2)}>`
            output.push(position.clone())
            let t = 0;
            if (Math.sign(direction.x) == 1) {
                t = 1;
            }
            let t_x = (block_x + step_size * t - position.x) / direction.x;

            if (t_x == Number.NEGATIVE_INFINITY || t_x == -0) {
                t_x = Number.POSITIVE_INFINITY
            }

            t = 0;
            if (Math.sign(direction.y) == 1) {
                t = 1;
            }
            let t_y = (this._floor_value(position.y, step_size) + step_size * Math.sign(direction.y) - position.y) / direction.y;
            t_y = (block_y + step_size * t - position.y) / direction.y;
            if (t_y == Number.NEGATIVE_INFINITY || t_y == -0) {
                t_y = Number.POSITIVE_INFINITY
            }
            if (t_x < 0) {
                debug_text += `<br>t_x negative: ${t_x}`
                break;

            }
            if (t_y < 0) {
                debug_text += `<br>t_y negative: ${t_y}`
                break;
            }
            if (t_y < t_x) {


                debug_text += `<br>t_y < t_x<br> ${t_y.toFixed(2)} < ${t_x.toFixed(2)}`;

                debug_text += `<br>step size: ${step_size}`
                position.x = position.x + t_y * direction.x;


                if (direction.y >= 0) {
                    block_y = block_y + step_size * Math.sign(direction.y);

                    position.y = position.y + t_y * direction.y;
                    if (this.isBlockPosInRange(this._floor_value(position.x, 1), block_y)) {
                        const next_step_size = this.getStepSize(this._floor_value(position.x, 1), block_y);
                        block_x = this._floor_value(position.x, next_step_size)
                        let node = this.getNode(block_x, block_y);
                        if (node.is_solid === true) {
                            const normal = new Vec2(0, -1.0 * Math.sign(direction.y));
                            debug_text += `<hr>NODE ${block_x}, ${block_y} is solid<br>normal: <${normal.x}, ${normal.y}>`
                            return [[position.clone(), new Vec2(block_x, block_y)], { "debug_text": debug_text }]
                        }
                    } else {
                        debug_text += "<br>done!"
                    }
                } else {
                    if (this.isBlockPosInRange(block_x, block_y - 1)) {

                        position.y = position.y + t_y * direction.y;
                        const next_step_size = this.getStepSize(this._floor_value(position.x, 1), block_y - 1);
                        block_x = this._floor_value(position.x, next_step_size);
                        block_y = block_y - next_step_size;
                        step_size = next_step_size;
                        let node = this.getNode(block_x, block_y);
                        if (node.is_solid === true) {
                            const normal = new Vec2(0, 1.0 * Math.sign(direction.y));
                            debug_text += `<hr>NODE ${block_x}, ${block_y} is solid<br>normal: <${normal.x}, ${normal.y}>`
                            return [[position.clone(), new Vec2(block_x, block_y)], { "debug_text": debug_text }]
                        }


                    } else {
                        debug_text += "<br>done!"
                        block_x = this._floor_value(position.x, step_size);
                        block_y = block_y - step_size;
                    }
                }
            } else {
                debug_text += `<br>t_x < t_y <br> ${t_x.toFixed(2)} < ${t_y.toFixed(2)}`;
                debug_text += `<br>step size: ${step_size}`
                position.y = position.y + t_x * direction.y;


                if (direction.x >= 0) {
                    block_x = block_x + step_size * Math.sign(direction.x);

                    position.x = position.x + t_x * direction.x;
                    if (this.isBlockPosInRange(block_x, this._floor_value(position.y, 1))) {
                        const next_step_size = this.getStepSize(block_x, this._floor_value(position.y, 1));
                        block_y = this._floor_value(position.y, next_step_size);
                        let node = this.getNode(block_x, block_y);
                        if (node.is_solid === true) {
                            const normal = new Vec2(-1.0 * Math.sign(direction.y), 0.);
                            debug_text += `<hr>NODE ${block_x}, ${block_y} is solid<br>normal: <${normal.x}, ${normal.y}>`
                            return [[position.clone(), new Vec2(block_x, block_y)], { "debug_text": debug_text }]
                        }
                    } else {
                        debug_text += "<br>done!"
                    }
                } else {

                    if (this.isBlockPosInRange(block_x - 1, block_y)) {
                        position.x = position.x + t_x * direction.x;

                        const next_step_size = this.getStepSize(block_x - 1, this._floor_value(position.y, 1));

                        block_y = this._floor_value(position.y, next_step_size);

                        block_x = block_x - next_step_size;
                        step_size = next_step_size;

                        let node = this.getNode(block_x, block_y);
                        if (node.is_solid === true) {
                            const normal = new Vec2(1.0 * Math.sign(direction.y), 0.);
                            debug_text += `<hr>NODE ${block_x}, ${block_y} is solid<br>normal: <${normal.x}, ${normal.y}>`
                            return [[position.clone(), new Vec2(block_x, block_y)], { "debug_text": debug_text }]
                        }

                    } else {
                        debug_text += "<br>done!"

                        block_x = block_x - step_size;
                        block_y = this._floor_value(position.y, step_size);


                    }
                }

            }
            debug_text += "<hr>"
        }
        return [output, { "debug_text": debug_text }]
    }
    getCollisions(origin, direction) {

        let debug_text = "";
        const max_side_length = Math.pow(2, this.size);
        const ray = new Ray(origin, direction);
        if (this.isRayOutside(ray)) {

            let t_x0 = -ray.origin.x / ray.direction.x;
            let b_x0 = null;
            if (t_x0 >= 0.) {
                const at = ray.at(t_x0);

                if (at.origin.y <= 0. || at.origin.y >= max_side_length) {
                    t_x0 = null;
                } else {
                    b_x0 = [0, Math.floor(at.origin.y)];
                }

            }
            let t_y0 = -ray.origin.y / ray.direction.y;
            let b_y0 = null;
            if (t_y0 >= 0.) {
                const at = ray.at(t_y0);

                if (at.origin.x <= 0. || at.origin.x >= max_side_length) {
                    t_y0 = null;
                } else {
                    b_y0 = [Math.floor(at.origin.x), 0]
                }


            }


            let t_x1 = (max_side_length - ray.origin.x) / ray.direction.x;
            let b_x1 = null;
            if (t_x1 >= 0.) {
                const at = ray.at(t_x1);

                if (at.origin.y <= 0. || at.origin.y >= max_side_length) {
                    t_x1 = null;
                } else {
                    b_x1 = [Math.pow(2, this.size) - 1, Math.floor(at.origin.y)]
                }
            }

            let t_y1 = (max_side_length - ray.origin.y) / ray.direction.y;
            let b_y1 = null;
            if (t_y1 >= 0.) {
                const at = ray.at(t_y1);

                if (at.origin.x <= 0. || at.origin.x >= max_side_length) {
                    t_y1 = null;

                } else {
                    b_y1 = [Math.floor(at.origin.x), Math.pow(2, this.size) - 1]
                }
            }

            let t_values = [
                { "t": t_x0, "block_pos": b_x0 },
                { "t": t_x1, "block_pos": b_x1 },
                { "t": t_y0, "block_pos": b_y0 },
                { "t": t_y1, "block_pos": b_y1 }
            ].filter((val) => val["t"] != null && val["block_pos"] != null && val["t"] != Infinity).filter((val) => (val["t"] >= 0.));
            const sorted = t_values.sort((a, b) => a["t"] >= b["t"]);

            const positions = sorted.map((value) => [ray.at(value["t"]).origin, value["block_pos"]]) ?? [];

            if (positions.length >= 1) {
                const first_collision = positions[0];
                const output_position = first_collision[0];
                if (first_collision[0].y == 16) {
                    debug_text += "16!!!\n"
                } else {
                    debug_text += `y: ${first_collision[0].y}`
                }

                debug_text += `block_pos: (${first_collision[1][0]},${first_collision[1][1]})`;


                const grid_x = first_collision[1][0];
                const grid_y = first_collision[1][1];
                let position = new Vec2(output_position.x, output_position.y);
                let direction = ray.direction;


                let step_size = this.getStepSize(grid_x, grid_y);



                let block_x = this._floor_value(grid_x, step_size);

                let block_y = this._floor_value(grid_y, step_size);

                debug_text += `block coords: <${block_x}, ${block_y}><br>step_size: ${step_size}`
                let output = [new Vec2(output_position.x, output_position.y)];
                output = []
                const block_history = []

                // starting infinite loop
                return this._rayIteration(block_x, block_y, position.clone(), direction, step_size)



            } else {
                return [[], { "debug_text": "" }]
            }

        } else {
            const grid_x = this._floor_value(ray.origin.x, 1)
            const grid_y = this._floor_value(ray.origin.y, 1)
            const step_size = this.getStepSize(grid_x, grid_y);
            const block_x = this._floor_value(grid_x, step_size)
            const block_y = this._floor_value(grid_y, step_size)
            return this._rayIteration(block_x, block_y, ray.origin.clone(), ray.direction.clone(), step_size)

        }

    }
    static getChildIndex(x, y) {
        if (x == 0 && y == 0) {
            return 0
        } else if (x == 0 && y == 1) {
            return 1
        } else if (x == 1 && y == 0) {
            return 2
        } else if (x == 1 && y == 1) {
            return 3
        } else {
            throw `Invalid x and y value, x: ${x}, y: ${y}`
        }
    }
    static childrenXY() {
        return [new Vec2(0, 0), new Vec2(0, 1), new Vec2(1, 0), new Vec2(1, 1)]
    }
    childrenXYWorld() {
        if (this.size <= 1) {
            //    throw "Can not look for children as size is 1"
        }
        return Node.childrenXY().map((value) => value.times(Math.pow(2, this.size - 1)))
    }
}
function event_to_world_vec(event) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left
    const y = event.clientY - rect.top;
    return Vec2.fromScreenXY(x, y)
}
const ROOT_BOX_SIZE = 20;
const BOX_SCREEN_ORIGIN = new Vec2(100., 100.);

let canvas = null;
function draw_circle(position, ctx, color, text = null) {
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(position.screenX(), position.screenY(), 10., 0., 2.0 * Math.PI);
    ctx.stroke();
    ctx.fillStyle = WritableStreamDefaultController
    ctx.fill();

    if (text != null) {
        ctx.strokeStyle = "#000";
        ctx.strokeText(text, position.screenX(), position.screenY())
    }

}
function main() {
    let cursor_pos = null;
    let clicked_cursor_pos = null;
    let running = true;
    canvas = document.getElementById("render_canvas");
    canvas.addEventListener("mousemove", (e) => {

        if (running) {
            cursor_pos = event_to_world_vec(e)
        }

    });
    canvas.addEventListener("mousedown", (event) => {
        if (running) {
            clicked_cursor_pos = event_to_world_vec(event);
        }



    })
    document.addEventListener("keydown", (e) => {
        if (e.key == ' ') {
            running = !running;
        }

    })



    const ctx = canvas.getContext("2d");
    const root_node = new Node(4);
    function draw() {
        debug_box = document.getElementById("debug_box");;
        debug_box.innerText = ""
        ctx.fillStyle = "#000000ff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        root_node.draw(ctx);

        if (cursor_pos != null) {
            if (clicked_cursor_pos == null) {
                draw_circle(cursor_pos, ctx, "#00ff00")
            } else {

                draw_circle(clicked_cursor_pos, ctx, "#0000ff");
                ctx.strokeStyle = "#0ff0aa";
                ctx.moveTo(cursor_pos.screenX(), cursor_pos.screenY());
                ctx.lineTo(clicked_cursor_pos.screenX(), clicked_cursor_pos.screenY());
                ctx.stroke();

                let [collisions, info] = root_node.getCollisions(clicked_cursor_pos, cursor_pos.minus(clicked_cursor_pos));
                let dom_element = document.getElementById("debug_box");
                dom_element.innerHTML = info["debug_text"];

                collisions.forEach((element, index) => {

                    let color = info["color"];
                    if (color == null) {
                        const colors = ["#0fffa0", "#003621ff", "#fbff00ff", "#757100ff"]
                        color = colors[colors.length - 1];
                        if (index < colors.length) {
                            color = colors[index];
                        }
                    }

                    draw_circle(element, ctx, color, text = index.toString())

                });
            }
        }


        requestAnimationFrame(draw);
    }
    requestAnimationFrame(draw)
}
