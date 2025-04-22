window.BENCHMARK_DATA = {
  "lastUpdate": 1745313971074,
  "repoUrl": "https://github.com/mripard/dradis",
  "entries": {
    "Dradis Benchmark": [
      {
        "commit": {
          "author": {
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d9903faa4086fa2e6cb10b717262fd10829c8db2",
          "message": "Merge pull request #201 from mripard/tracing-improvements\n\nAdd more traces and create some benchmarks, with CI",
          "timestamp": "2025-03-28T14:30:07+01:00",
          "tree_id": "dfcc635e6f84ac02f5b4b4a290c43312c6bef357",
          "url": "https://github.com/mripard/dradis/commit/d9903faa4086fa2e6cb10b717262fd10829c8db2"
        },
        "date": 1743168885972,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 8459380,
            "range": "± 9153",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ea81c8032a34dd1223c33928a29f0ec308ac91d4",
          "message": "Merge pull request #202 from mripard/switch-to-rxing\n\ndradis: Switch to rxing",
          "timestamp": "2025-03-28T14:47:11+01:00",
          "tree_id": "c2f21fd5efe03596a77dd05a9d3a69323488c8cf",
          "url": "https://github.com/mripard/dradis/commit/ea81c8032a34dd1223c33928a29f0ec308ac91d4"
        },
        "date": 1743169905075,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 853738,
            "range": "± 8699",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a76d4202f6970d2f1eadb0c38d38b85cfecee74c",
          "message": "Merge pull request #203 from mripard/ci-bench-comment-always\n\ngithub: Always report perf difference",
          "timestamp": "2025-03-28T16:46:00+01:00",
          "tree_id": "495b3018b1b3e514a4ba5d9754ff122d038575d7",
          "url": "https://github.com/mripard/dradis/commit/a76d4202f6970d2f1eadb0c38d38b85cfecee74c"
        },
        "date": 1743177114902,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 1775181,
            "range": "± 16006",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d36d070ced7df710f6935a449bb32fb746e5311",
          "message": "Merge pull request #206 from mripard/dump-faulty-frames\n\nDump Faulty Frames",
          "timestamp": "2025-04-22T11:22:57+02:00",
          "tree_id": "dffe64b58a99e97f04c1e9fce70fd44d7ef2e06f",
          "url": "https://github.com/mripard/dradis/commit/5d36d070ced7df710f6935a449bb32fb746e5311"
        },
        "date": 1745313970432,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 489497,
            "range": "± 2195",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}