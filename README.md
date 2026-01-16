# rbush-rs
A high-performance(>5x faster), 100% Rust port of [RBush](https://github.com/mourner/rbush) spatial index library, compiled to WebAssembly.

`rbush-rs` is designed to be a near drop-in replacement for `rbush`, offering significant performance improvements for bulk loading and searching massive datasets.

```diff
+ Import { RBush } from 'rbush-rs';
- Import { RBush } from 'rbush';
```

## Benchmark

Tests performed on a dataset of 10,000 rectangles.

| Operation | JS (rbush) | WASM (rbush-rs) | Speedup |
| --- | --- | --- | --- |
| **Bulk Load (Hybrid)** | 20.23 ms | **3.60 ms** | **~5.6x Faster** |
| **Insert (1000 items)** | 36.58 ms | **4.04 ms** | **~9.1x Faster** |
| **Remove (1000 items)** | 58.03 ms | **7.85 ms** | **~7.4x Faster** |
| **Search** | 0.042 ms | **0.027 ms** | **~1.6x Faster** |
| **Collides** | 0.003 ms | 0.003 ms | **1x Faster** |
| **Clear** | 0.001 ms | 0.001 ms | **1x Faster** |

*\*Standard load is slower due to the overhead of reading JS objects into WASM. Use Hybrid Load for maximum performance.*

## Installation
You can install `rbush-rs` via:

### npm
```bash
npm install rbush-rs
```

### pnpm
```bash
pnpm install rbush-rs
```

### yarn
```bash
yarn add rbush-rs
```


## Usage

### The "Drop-in" Replacement

Use this mode if you want to switch libraries with minimal code changes. It accepts the same data format as the original `rbush`.

```javascript
import { RBush } from 'rbush-rs';

// Initialize (max entries per node, default 9)
const tree = new RBush(9);

// Load standard data (Array of objects with minX, minY, maxX, maxY)
const items = [
    { minX: 10, minY: 10, maxX: 20, maxY: 20, id: 'a' },
    { minX: 50, minY: 50, maxX: 60, maxY: 60, id: 'b' }
];
tree.load(items);

// Search
const results = tree.search({ minX: 0, minY: 0, maxX: 30, maxY: 30 });
console.log(results); // [{ minX: 10, ... }]

```

### The "Hybrid" Load (Recommended for MAX Speed)

To unlock the 5-6x load speedup, pass the coordinates as a `Float64Array` alongside your items. This allows Rust to sort the data instantly without expensive JavaScript object lookups.

```javascript
import { RBush } from 'rbush-rs';

const tree = new RBush(9);
const items = [];
const count = 10000;

// Create a flat array for coordinates [minX, minY, maxX, maxY, minX...]
// Size = count * 4
const flatCoords = new Float64Array(count * 4);

for(let i = 0; i < count; i++) {
    const item = { minX: Math.random() * 100, minY: Math.random() * 100, maxX: ..., maxY: ..., id: i };
    
    // standard items array
    items.push(item);

    // flat array
    flatCoords[i*4]     = item.minX;
    flatCoords[i*4 + 1] = item.minY;
    flatCoords[i*4 + 2] = item.maxX;
    flatCoords[i*4 + 3] = item.maxY;
}

// Load both at once
tree.loadHybrid(flatCoords, items);

// Search is exactly the same
const results = tree.search({ minX: 0, minY: 0, maxX: 50, maxY: 50 });

```

### Other Operations

All operations below work regardless of how you loaded the data (Standard or Hybrid).

#### Single Insertion

`rbush-rs` is highly optimized for dynamic updates, performing ~9x faster than JS.

```javascript
const item = { minX: 20, minY: 20, maxX: 30, maxY: 30, id: 'c' };
tree.insert(item);
```

#### Removal

Removal is ~7x faster than JS. The item object passed must match the one in the tree (by reference equality).

```javascript
tree.remove(item);
```

#### Collision Detection

Checks if there are any items in the bounding box. Faster than `search` if you don't need the actual items.

```javascript
const hasCollision = tree.collides({ minX: 10, minY: 10, maxX: 20, maxY: 20 });
// returns true or false
```

#### Retrieve All Items

Returns all items currently stored in the tree.

```javascript
const allItems = tree.all();
```

#### Clear Tree

Removes all items and resets the tree.

```javascript
tree.clear();
```

## 🔧 API Reference

* **`new RBush(maxEntries?: number)`**: Creates a new tree.
* **`load(items: array)`**: Bulk loads standard JS objects.
* **`loadHybrid(coords: Float64Array, items: array)`**: High-performance bulk load.
* **`insert(item: object)`**: Inserts a single item.
* **`remove(item: object)`**: Removes a specific item.
* **`search(bbox: object)`**: Returns an array of items intersecting the bbox.
* **`collides(bbox: object)`**: Returns `true` if any item intersects the bbox.
* **`all()`**: Returns all items in the tree.
* **`clear()`**: Removes all items.