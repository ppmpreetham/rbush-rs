import { RBush as WasmRBush } from './rbush_rs.js';

export default class RBush {
    constructor(maxEntries = 9) {
        this._tree = new WasmRBush(maxEntries);
    }

    toBBox(item) { return item; }

    insert(item) {
        const b = this.toBBox(item);
        this._tree.insert({ 
            ...item, 
            minX: b.minX, minY: b.minY, maxX: b.maxX, maxY: b.maxY 
        });
        return this;
    }

    load(data) {
        const normalized = data.map(item => {
            const b = this.toBBox(item);
            return { ...item, minX: b.minX, minY: b.minY, maxX: b.maxX, maxY: b.maxY };
        });
        this._tree.load(normalized);
        return this;
    }

    remove(item) {
        const b = this.toBBox(item);
        this._tree.remove({ ...item, minX: b.minX, minY: b.minY, maxX: b.maxX, maxY: b.maxY });
        return this;
    }

    search(bbox) { return this._tree.search(bbox); }
    collides(bbox) { return this._tree.collides(bbox); }
    all() { return this._tree.all(); }
    clear() { this._tree.clear(); return this; }
    toJSON() { return this._tree.toJSON(); }
    fromJSON(data) { this._tree.fromJSON(data); return this; }
    
    destroy() { this._tree.free(); }
}