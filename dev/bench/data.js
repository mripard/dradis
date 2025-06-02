window.BENCHMARK_DATA = {
  "lastUpdate": 1748867273080,
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
          "id": "0a127170345ae551c9a2e3092739267bce7f70e4",
          "message": "Merge pull request #207 from mripard/update-v4lise\n\nConsolidate v4lise crates",
          "timestamp": "2025-04-22T15:21:09+02:00",
          "tree_id": "e0b3c519dce52ba50430fe2bc1e62deeff3ee098",
          "url": "https://github.com/mripard/dradis/commit/0a127170345ae551c9a2e3092739267bce7f70e4"
        },
        "date": 1745328279869,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 492471,
            "range": "± 4155",
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
          "id": "fafe765cc543c92c15849ad593026896a5e7589c",
          "message": "Merge pull request #208 from mripard/update-v4lise\n\ndradis: Update crates",
          "timestamp": "2025-04-22T15:37:39+02:00",
          "tree_id": "9df648e72138d61bd63680d9357de137b665650f",
          "url": "https://github.com/mripard/dradis/commit/fafe765cc543c92c15849ad593026896a5e7589c"
        },
        "date": 1745329250293,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 490399,
            "range": "± 1635",
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
          "id": "29d61b25bd3bafc201fef14eb97466744f676f16",
          "message": "Merge pull request #209 from mripard/lints-dradis\n\nEnable More Lints",
          "timestamp": "2025-04-23T10:49:43+02:00",
          "tree_id": "89ed05a49902a5544102bbd7e18eebc02125cee0",
          "url": "https://github.com/mripard/dradis/commit/29d61b25bd3bafc201fef14eb97466744f676f16"
        },
        "date": 1745398373007,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 501844,
            "range": "± 4836",
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
          "id": "c599f9431fc07cf931d00b38806754977c5bed43",
          "message": "Merge pull request #210 from mripard/dependabot/cargo/nix-0.30.0\n\nbuild(deps): update nix requirement from 0.29.0 to 0.30.0",
          "timestamp": "2025-05-01T19:09:26+02:00",
          "tree_id": "45ea0e0d7c0ce59ab71835c775dc6219a611f265",
          "url": "https://github.com/mripard/dradis/commit/c599f9431fc07cf931d00b38806754977c5bed43"
        },
        "date": 1746119566334,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 482838,
            "range": "± 1448",
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
          "id": "ec83ac27d27898fabaed15658d7b2faab79514ca",
          "message": "Merge pull request #211 from mripard/media-framework-support\n\nMedia Pipeline Support",
          "timestamp": "2025-05-27T11:26:42+02:00",
          "tree_id": "0e31ec10a943870dbdc21dbd782a6dfede678079",
          "url": "https://github.com/mripard/dradis/commit/ec83ac27d27898fabaed15658d7b2faab79514ca"
        },
        "date": 1748338280685,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 520527,
            "range": "± 1658",
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
          "id": "2797eaf6f2b5adfb3833b74224e5bdbafe23876f",
          "message": "Merge pull request #216 from mripard/dependabot-fixes\n\nDependabot fixes",
          "timestamp": "2025-06-02T14:23:38+02:00",
          "tree_id": "59fc0bb3b00000794e973b8b784e6638c1908fea",
          "url": "https://github.com/mripard/dradis/commit/2797eaf6f2b5adfb3833b74224e5bdbafe23876f"
        },
        "date": 1748867272470,
        "tool": "cargo",
        "benches": [
          {
            "name": "frame processing/whole",
            "value": 490906,
            "range": "± 2019",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}