const RBushJS = require("rbush")
const { RBush: RBushWasm } = require("./pkg/rbush_rs")

describe("RBush Comprehensive Benchmark", () => {
  function generateData(count) {
    const data = []
    const flatData = new Float64Array(count * 4)
    for (let i = 0; i < count; i++) {
      const minX = Math.random() * 1000
      const minY = Math.random() * 1000
      const maxX = minX + Math.random() * 20
      const maxY = minY + Math.random() * 20

      data.push({ minX, minY, maxX, maxY, id: i })

      flatData[i * 4] = minX
      flatData[i * 4 + 1] = minY
      flatData[i * 4 + 2] = maxX
      flatData[i * 4 + 3] = maxY
    }
    return { data, flatData }
  }

  const DATA_SIZE = 10000
  const { data: testData, flatData: testDataFlat } = generateData(DATA_SIZE)
  const searchBox = { minX: 400, minY: 400, maxX: 600, maxY: 600 }
  const itemsToRemove = testData.slice(0, 1000)

  function benchmark(name, fn, iterations = 100) {
    const start = performance.now()
    for (let i = 0; i < iterations; i++) {
      fn()
    }
    const end = performance.now()
    const avg = (end - start) / iterations
    console.log(`${name.padEnd(30)}: ${avg.toFixed(3)} ms`)
  }

  console.log(`\nBENCHMARK RESULTS (N=${DATA_SIZE})`)

  test("Benchmark: Bulk Load", () => {
    console.log("\n Bulk Load ")
    benchmark(
      "JS RBush",
      () => {
        const tree = new RBushJS(9)
        tree.load(testData)
      },
      10
    )

    benchmark(
      "WASM RBush (Hybrid)",
      () => {
        const tree = new RBushWasm(9)
        tree.loadHybrid(testDataFlat, testData)
      },
      10
    )
  })

  test("Benchmark: Insert (One by One)", () => {
    console.log("\n Insert (1000 items) ")

    const smallSet = testData.slice(0, 1000)

    benchmark(
      "JS RBush",
      () => {
        const tree = new RBushJS(9)
        for (const item of smallSet) tree.insert(item)
      },
      20
    )

    benchmark(
      "WASM RBush",
      () => {
        const tree = new RBushWasm(9)
        for (const item of smallSet) tree.insert(item)
      },
      20
    )
  })

  test("Benchmark: Search", () => {
    console.log("\n Search ")
    const jsTree = new RBushJS(9)
    jsTree.load(testData)

    const wasmTree = new RBushWasm(9)
    wasmTree.loadHybrid(testDataFlat, testData)

    benchmark("JS RBush", () => {
      jsTree.search(searchBox)
    })

    benchmark("WASM RBush", () => {
      wasmTree.search(searchBox)
    })
  })

  test("Benchmark: Collides", () => {
    console.log("\n Collides ")
    const jsTree = new RBushJS(9)
    jsTree.load(testData)

    const wasmTree = new RBushWasm(9)
    wasmTree.loadHybrid(testDataFlat, testData)

    benchmark("JS RBush", () => {
      jsTree.collides(searchBox)
    })

    benchmark("WASM RBush", () => {
      wasmTree.collides(searchBox)
    })
  })

  test("Benchmark: Remove", () => {
    console.log("\n Remove (1000 items) ")

    benchmark(
      "JS RBush",
      () => {
        const tree = new RBushJS(9)
        tree.load(testData)
        for (const item of itemsToRemove) tree.remove(item)
      },
      5
    )

    benchmark(
      "WASM RBush",
      () => {
        const tree = new RBushWasm(9)
        tree.loadHybrid(testDataFlat, testData)
        for (const item of itemsToRemove) tree.remove(item)
      },
      5
    )
  })

  test("Benchmark: Clear", () => {
    console.log("\n Clear ")
    const jsTree = new RBushJS(9)
    jsTree.load(testData)

    const wasmTree = new RBushWasm(9)
    wasmTree.loadHybrid(testDataFlat, testData)

    benchmark(
      "JS RBush",
      () => {
        jsTree.clear()
      },
      1000
    )

    benchmark(
      "WASM RBush",
      () => {
        wasmTree.clear()
      },
      1000
    )
  })
})
