window.BENCHMARK_DATA = {
  "lastUpdate": 1760621883163,
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
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
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 490906,
            "range": "± 2019",
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
          "id": "b137eaf76fefd8a33cc18d8d5ba0df1b43476d74",
          "message": "Merge pull request #215 from mripard/dependabot/cargo/facet-0.27.8\n\nbuild(deps): update facet requirement from 0.25.1 to 0.27.8",
          "timestamp": "2025-06-02T14:36:06+02:00",
          "tree_id": "d6253ddac7bf7b642ebeff88ba7242e1fe456639",
          "url": "https://github.com/mripard/dradis/commit/b137eaf76fefd8a33cc18d8d5ba0df1b43476d74"
        },
        "date": 1748868012787,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 491528,
            "range": "± 2222",
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
          "id": "4634f71b4ce46e2cd8b55cd52f8e600171a5dc2b",
          "message": "Merge pull request #214 from mripard/dependabot/cargo/criterion-0.6.0\n\nbuild(deps): update criterion requirement from 0.5.1 to 0.6.0",
          "timestamp": "2025-06-02T14:47:41+02:00",
          "tree_id": "179ad58df1c7fa72ee466a264811823ece339576",
          "url": "https://github.com/mripard/dradis/commit/4634f71b4ce46e2cd8b55cd52f8e600171a5dc2b"
        },
        "date": 1748868703222,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 499695,
            "range": "± 2062",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "b32740964b9d761bd3c5df298aa916f122dc6529",
          "message": "dependabot: fix facet pattern\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-02T17:02:52+02:00",
          "tree_id": "fc62d3f753dbef74c7a096c02f78bd394195cc35",
          "url": "https://github.com/mripard/dradis/commit/b32740964b9d761bd3c5df298aa916f122dc6529"
        },
        "date": 1748876784968,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 486103,
            "range": "± 5360",
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
          "id": "af9254c55559bb9f3f46ede70a28fec8c4127413",
          "message": "Merge pull request #218 from mripard/dradis-import\n\nImport Boomer into the repo",
          "timestamp": "2025-06-03T14:03:02+02:00",
          "tree_id": "3a9b50a8b78a504b8cc32ef84875691461ba49a8",
          "url": "https://github.com/mripard/dradis/commit/af9254c55559bb9f3f46ede70a28fec8c4127413"
        },
        "date": 1748952421790,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 483711,
            "range": "± 1611",
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
          "id": "fc77962a54f3f4a422e1e4acdfe8b6c41bdd07f5",
          "message": "Merge pull request #219 from mripard/updates\n\nFurther Boomer Improvements",
          "timestamp": "2025-06-04T14:07:59+02:00",
          "tree_id": "4fdad72e836cc715ec8fd0dab0631e2ec9761698",
          "url": "https://github.com/mripard/dradis/commit/fc77962a54f3f4a422e1e4acdfe8b6c41bdd07f5"
        },
        "date": 1749039166562,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 496103,
            "range": "± 4230",
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
          "id": "12502590e0a385747af0573d09c1a9c5d8db5c57",
          "message": "Merge pull request #220 from mripard/better-tracing-fixes\n\nImprove tracing, and several fixes",
          "timestamp": "2025-06-13T10:40:00+02:00",
          "tree_id": "a0c1b7974e217351847545eaa2db3cdd3f0d4625",
          "url": "https://github.com/mripard/dradis/commit/12502590e0a385747af0573d09c1a9c5d8db5c57"
        },
        "date": 1749804256836,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 473598,
            "range": "± 1400",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "1ab95b73e6abe9aec0d095ae42278d9f4f6ff24f",
          "message": "chore: Release",
          "timestamp": "2025-06-13T10:45:44+02:00",
          "tree_id": "b033e210574eb0d086b451644f4e69ece987c370",
          "url": "https://github.com/mripard/dradis/commit/1ab95b73e6abe9aec0d095ae42278d9f4f6ff24f"
        },
        "date": 1749804610862,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 488605,
            "range": "± 2886",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "8c6b1fbf6fd683a218798118d113ab6d2ea5c24b",
          "message": "github: Rework release scripts",
          "timestamp": "2025-06-13T11:25:32+02:00",
          "tree_id": "4f6fdc8d81e7656859dd24377fc61a5ec00bbdf2",
          "url": "https://github.com/mripard/dradis/commit/8c6b1fbf6fd683a218798118d113ab6d2ea5c24b"
        },
        "date": 1749807005575,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 482871,
            "range": "± 923",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "8c6b1fbf6fd683a218798118d113ab6d2ea5c24b",
          "message": "github: Rework release scripts",
          "timestamp": "2025-06-13T11:25:32+02:00",
          "tree_id": "4f6fdc8d81e7656859dd24377fc61a5ec00bbdf2",
          "url": "https://github.com/mripard/dradis/commit/8c6b1fbf6fd683a218798118d113ab6d2ea5c24b"
        },
        "date": 1749807433503,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 486556,
            "range": "± 2822",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "b76f1b38c030eb2c38a873c3c81db9603d2412f8",
          "message": "github: Dump context to logs\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-13T11:38:10+02:00",
          "tree_id": "0bbb0ab53db76e8938e80cf46a3085cc87567b55",
          "url": "https://github.com/mripard/dradis/commit/b76f1b38c030eb2c38a873c3c81db9603d2412f8"
        },
        "date": 1749807740085,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 481712,
            "range": "± 1815",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "1e086f13a59255da285dcef7f45aea4305c450a4",
          "message": "github: Fix if clause (hopefully)\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-13T11:44:37+02:00",
          "tree_id": "9c6f584a337c48f29870d16100882541fde64f81",
          "url": "https://github.com/mripard/dradis/commit/1e086f13a59255da285dcef7f45aea4305c450a4"
        },
        "date": 1749808138805,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 477703,
            "range": "± 1704",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "1037614710452b7892cb224134dd2c68e680373b",
          "message": "github: Provide default if PR number isn't set\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-13T11:59:31+02:00",
          "tree_id": "62cd985ba8af396e39044cf03b71520f515e5c90",
          "url": "https://github.com/mripard/dradis/commit/1037614710452b7892cb224134dd2c68e680373b"
        },
        "date": 1749809057942,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 491293,
            "range": "± 3868",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "091ddf28dec132c78daa6c41d58e53c471703722",
          "message": "chore: Release",
          "timestamp": "2025-06-13T12:05:23+02:00",
          "tree_id": "ea8f4670041fe67ef89b9c0340e69ac15be7da06",
          "url": "https://github.com/mripard/dradis/commit/091ddf28dec132c78daa6c41d58e53c471703722"
        },
        "date": 1749809390664,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 488841,
            "range": "± 3128",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "c4eea05929ac03e33928bebeae081ce8f30125e3",
          "message": "github: Stop dropping context\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-13T12:06:48+02:00",
          "tree_id": "44f57e8c1587ae9b2820fac55066c89788539f28",
          "url": "https://github.com/mripard/dradis/commit/c4eea05929ac03e33928bebeae081ce8f30125e3"
        },
        "date": 1749809433532,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 485921,
            "range": "± 3414",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "aa5bc7881c7e594d3f41497f4818387fc08c022e",
          "message": "cargo: Add Cargo.lock\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-13T13:26:07+02:00",
          "tree_id": "7ee0ede04e827441993643a2551c3913c183dbd9",
          "url": "https://github.com/mripard/dradis/commit/aa5bc7881c7e594d3f41497f4818387fc08c022e"
        },
        "date": 1749814221524,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 484953,
            "range": "± 3347",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "ee5c01eba1372f31e87b6530a6d739476f52b50b",
          "message": "github: Add Release Notes\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-06-13T15:54:58+02:00",
          "tree_id": "7c932665023358207e98ced2348a70c592da2189",
          "url": "https://github.com/mripard/dradis/commit/ee5c01eba1372f31e87b6530a6d739476f52b50b"
        },
        "date": 1749823159005,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 479272,
            "range": "± 1666",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "2d9197c9f7c407ed7d1b6000285cefab89c85be1",
          "message": "chore: Release",
          "timestamp": "2025-06-13T15:57:36+02:00",
          "tree_id": "332da774a425a318ad54a3f2a1a743decadc2740",
          "url": "https://github.com/mripard/dradis/commit/2d9197c9f7c407ed7d1b6000285cefab89c85be1"
        },
        "date": 1749823337212,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 488417,
            "range": "± 3720",
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
          "id": "80bbb2a26b045636ee7e3c22185eb0f84a29633d",
          "message": "Merge pull request #221 from mripard/dradis-fix-rgb888\n\nv4l2-raw: Reintroduce RGB24",
          "timestamp": "2025-06-16T15:39:06+02:00",
          "tree_id": "2ce70e1c440cdf2574dfe3de641322676f5260c5",
          "url": "https://github.com/mripard/dradis/commit/80bbb2a26b045636ee7e3c22185eb0f84a29633d"
        },
        "date": 1750081393757,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 484338,
            "range": "± 18584",
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
          "id": "f72d44759bc4edb9357715920f4eb733c3c0357b",
          "message": "Merge pull request #223 from JoseExposito/readme-broken-links\n\nREADME: Update links",
          "timestamp": "2025-06-25T16:33:36+02:00",
          "tree_id": "e38b820550a0d6a5a142e452044c3bc82de795f9",
          "url": "https://github.com/mripard/dradis/commit/f72d44759bc4edb9357715920f4eb733c3c0357b"
        },
        "date": 1750862306756,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 478937,
            "range": "± 1265",
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
          "id": "b61ac7a98e5b61bd84f41f5a2fbc3278d7d85932",
          "message": "Merge pull request #224 from JoseExposito/select-connector\n\nAllow to select connector",
          "timestamp": "2025-06-26T08:57:52+02:00",
          "tree_id": "9dc48ab25414faf22bbc886e9f3d9ef6b756afd1",
          "url": "https://github.com/mripard/dradis/commit/b61ac7a98e5b61bd84f41f5a2fbc3278d7d85932"
        },
        "date": 1750921361610,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 484293,
            "range": "± 2264",
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
          "id": "f1d95fe2415c3bcadad53b439faa825fc3492151",
          "message": "Merge pull request #222 from mripard/dump-fixes\n\nImprove the logic to dump frames",
          "timestamp": "2025-06-26T08:58:53+02:00",
          "tree_id": "ac08c5d35746a423947dd75e9a18d24e1e96b700",
          "url": "https://github.com/mripard/dradis/commit/f1d95fe2415c3bcadad53b439faa825fc3492151"
        },
        "date": 1750921367385,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 491047,
            "range": "± 3225",
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
          "id": "51892122df412d7b264cdcb909dc2c3b982bd734",
          "message": "Merge pull request #225 from mripard/boomer-plane-fix\n\nboomer: Use Primary planes instead of overlays",
          "timestamp": "2025-06-26T09:33:10+02:00",
          "tree_id": "b50bcd741c24c755bf89cc66629fc72c8ebdd99a",
          "url": "https://github.com/mripard/dradis/commit/51892122df412d7b264cdcb909dc2c3b982bd734"
        },
        "date": 1750923524678,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 494743,
            "range": "± 3877",
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
          "id": "bcab80597006d784bd2bd479616b70fe3031221d",
          "message": "Merge pull request #226 from JoseExposito/log-output\n\nboomer: Log output information",
          "timestamp": "2025-06-30T12:49:53+02:00",
          "tree_id": "a3ea5e605c6147a46ba38f873c89526462c19fbd",
          "url": "https://github.com/mripard/dradis/commit/bcab80597006d784bd2bd479616b70fe3031221d"
        },
        "date": 1751280862348,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 482919,
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
          "id": "f55525f2ba2b09c925bd6d1a46e34a6e66cd7c89",
          "message": "Merge pull request #232 from mripard/ci-updates\n\n.github: Update workflows",
          "timestamp": "2025-07-01T10:59:59+02:00",
          "tree_id": "323a75d7622e13827681a4ff233b9622df1dfa8e",
          "url": "https://github.com/mripard/dradis/commit/f55525f2ba2b09c925bd6d1a46e34a6e66cd7c89"
        },
        "date": 1751360620759,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 481901,
            "range": "± 2321",
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
          "id": "def43d5c99b20f50605d2ac43f0580d17bd9d0e4",
          "message": "Merge pull request #231 from mripard/dependabot/cargo/serde_with-3.14.0\n\nbuild(deps): bump serde_with from 3.12.0 to 3.14.0",
          "timestamp": "2025-07-01T11:01:05+02:00",
          "tree_id": "36171f185b8238deb05355ffb46e4c42ed3c1a99",
          "url": "https://github.com/mripard/dradis/commit/def43d5c99b20f50605d2ac43f0580d17bd9d0e4"
        },
        "date": 1751360723626,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 481794,
            "range": "± 2156",
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
          "id": "faa875412f1d47d6b6897341a0de926bcf789c3a",
          "message": "Merge pull request #228 from mripard/dependabot/cargo/tracelimit-01d7560\n\nbuild(deps): bump tracelimit from `3c2dda8` to `01d7560`",
          "timestamp": "2025-07-01T11:01:44+02:00",
          "tree_id": "80875f926ed85ff0dab171da06ddeec689d0c7e8",
          "url": "https://github.com/mripard/dradis/commit/faa875412f1d47d6b6897341a0de926bcf789c3a"
        },
        "date": 1751360764267,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 488708,
            "range": "± 13008",
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
          "id": "bda7d5e24b830470877981c7c5dddcb8f0d31964",
          "message": "Merge pull request #230 from mripard/dependabot/cargo/bindgen-0.72.0\n\nbuild(deps): bump bindgen from 0.71.1 to 0.72.0",
          "timestamp": "2025-07-01T11:06:33+02:00",
          "tree_id": "609c6aa35b5ed0a20aae1898314addd25a4bc6e4",
          "url": "https://github.com/mripard/dradis/commit/bda7d5e24b830470877981c7c5dddcb8f0d31964"
        },
        "date": 1751361068910,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 480710,
            "range": "± 2026",
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
          "id": "5c8fba23e91f92599af91d3516517cd0cf635dff",
          "message": "Merge pull request #227 from JoseExposito/unwrap-plane-type\n\nboomer: Unwrap plane.plane_type()",
          "timestamp": "2025-07-01T11:20:17+02:00",
          "tree_id": "72153d3dfa7a1cc884b861b497002dd2ef5da1e7",
          "url": "https://github.com/mripard/dradis/commit/5c8fba23e91f92599af91d3516517cd0cf635dff"
        },
        "date": 1751361869902,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 492638,
            "range": "± 3390",
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
          "id": "2ccff037dce4e7a3bd89dad43e5c4003d76f6f6d",
          "message": "Merge pull request #229 from mripard/dependabot/cargo/redid-862de87\n\nbuild(deps): bump redid from `4e966cc` to `862de87`",
          "timestamp": "2025-07-01T17:41:56+02:00",
          "tree_id": "57e0333a57f6cb01c335336c1bc5176fe3e14574",
          "url": "https://github.com/mripard/dradis/commit/2ccff037dce4e7a3bd89dad43e5c4003d76f6f6d"
        },
        "date": 1751384767198,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 482447,
            "range": "± 1281",
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
          "id": "1bab1d1040aff80ac3637f9d91df4964e8e8593b",
          "message": "Merge pull request #235 from JoseExposito/gitignore-dumped-buffers\n\ngitignore: Ignore dumped buffers",
          "timestamp": "2025-07-02T08:02:32+02:00",
          "tree_id": "6a7fddcdda58a5b83c4012ea58893d9a273f809c",
          "url": "https://github.com/mripard/dradis/commit/1bab1d1040aff80ac3637f9d91df4964e8e8593b"
        },
        "date": 1751436398194,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 480798,
            "range": "± 1865",
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
          "id": "42c5a41743c4c4c711e4999d0216ea8c14d6c996",
          "message": "Merge pull request #233 from mripard/fix-used-buffer-bytes\n\ndradis: Only access the memory actually used",
          "timestamp": "2025-07-02T09:38:42+02:00",
          "tree_id": "5b8e4b19ad98369cad239d3f4b670b7889b4b5cc",
          "url": "https://github.com/mripard/dradis/commit/42c5a41743c4c4c711e4999d0216ea8c14d6c996"
        },
        "date": 1751442161102,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 491445,
            "range": "± 2281",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "2f2b660b09a2247eac1b51e5c5119debdfd161a6",
          "message": "github: fix benchmark permissions\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-07-02T13:41:25+02:00",
          "tree_id": "422d91799e31d63b97e2ef70aa036ffecc707318",
          "url": "https://github.com/mripard/dradis/commit/2f2b660b09a2247eac1b51e5c5119debdfd161a6"
        },
        "date": 1751456749927,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 485605,
            "range": "± 2266",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "d517c2435f9105005dbd795a02f3126c07ad1e62",
          "message": "Update README\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-07-04T16:09:02+02:00",
          "tree_id": "8a7ac932702a497e35deca2891fec58a28eb4ccf",
          "url": "https://github.com/mripard/dradis/commit/d517c2435f9105005dbd795a02f3126c07ad1e62"
        },
        "date": 1751638397838,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 487269,
            "range": "± 2775",
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
          "id": "be32e75b5012f10d973684fb936656f98f29c92a",
          "message": "Merge pull request #239 from JoseExposito/clippy-error-dtd.vbp\n\ndradis: Fix clippy error",
          "timestamp": "2025-07-15T14:07:08+02:00",
          "tree_id": "51bfe4a9bb8ccd7260e63b63431d653e9462c3cb",
          "url": "https://github.com/mripard/dradis/commit/be32e75b5012f10d973684fb936656f98f29c92a"
        },
        "date": 1752581458611,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 479550,
            "range": "± 1656",
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
          "id": "5819bd0590c492c6d953abcf0a703c9d6bd11481",
          "message": "Merge pull request #251 from mripard/dependabot/cargo/rustix-1.0.8\n\nbuild(deps): bump rustix from 1.0.7 to 1.0.8",
          "timestamp": "2025-08-01T14:06:05+02:00",
          "tree_id": "b8f57b77947d485664262b5d44edf47ac357ddca",
          "url": "https://github.com/mripard/dradis/commit/5819bd0590c492c6d953abcf0a703c9d6bd11481"
        },
        "date": 1754050202168,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 486160,
            "range": "± 2547",
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
          "id": "4a015717ce6b6539e8a191bf494d46716fd82e23",
          "message": "Merge pull request #248 from mripard/dependabot/cargo/clap-4.5.42\n\nbuild(deps): bump clap from 4.5.40 to 4.5.42",
          "timestamp": "2025-08-01T14:07:06+02:00",
          "tree_id": "d524c4ea5bb1eb123912ab446483cae463c9684b",
          "url": "https://github.com/mripard/dradis/commit/4a015717ce6b6539e8a191bf494d46716fd82e23"
        },
        "date": 1754050230329,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 503293,
            "range": "± 2380",
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
          "id": "e7a7399477ad47ec04329a8c6f292e0a45045638",
          "message": "Merge pull request #249 from mripard/dependabot/cargo/serde_json-1.0.142\n\nbuild(deps): bump serde_json from 1.0.140 to 1.0.142",
          "timestamp": "2025-08-01T14:06:37+02:00",
          "tree_id": "7b8ecbf9328246630839996b53125cd4a647c923",
          "url": "https://github.com/mripard/dradis/commit/e7a7399477ad47ec04329a8c6f292e0a45045638"
        },
        "date": 1754050230587,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 499968,
            "range": "± 2601",
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
          "id": "f782369c09e21c68f544403a1a78915b012d405f",
          "message": "Merge pull request #250 from mripard/dependabot/cargo/tracelimit-53956a2\n\nbuild(deps): bump tracelimit from `01d7560` to `53956a2`",
          "timestamp": "2025-08-01T14:06:53+02:00",
          "tree_id": "2b44c02e95f8ab52f121e11a981f6835b225b009",
          "url": "https://github.com/mripard/dradis/commit/f782369c09e21c68f544403a1a78915b012d405f"
        },
        "date": 1754050240091,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 497489,
            "range": "± 4213",
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
          "id": "e7b0efb0d95cc29a542e5c6a117160d44ed9aedf",
          "message": "Merge pull request #252 from mripard/dependabot/cargo/serde_json-1.0.143\n\nbuild(deps): bump serde_json from 1.0.142 to 1.0.143",
          "timestamp": "2025-08-19T12:01:37+02:00",
          "tree_id": "3d0b98e1b5d193a86d4725ded29ccfa991eda07b",
          "url": "https://github.com/mripard/dradis/commit/e7b0efb0d95cc29a542e5c6a117160d44ed9aedf"
        },
        "date": 1755597919624,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 490406,
            "range": "± 6629",
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
          "id": "c637f5807060300f0ba30aee39547cf15fe80779",
          "message": "Merge pull request #253 from mripard/dependabot/cargo/tracelimit-b9a2abc\n\nbuild(deps): bump tracelimit from `53956a2` to `b9a2abc`",
          "timestamp": "2025-08-19T12:02:03+02:00",
          "tree_id": "ea2f105c9d8c392e096ec902262a633ebbdf7d53",
          "url": "https://github.com/mripard/dradis/commit/c637f5807060300f0ba30aee39547cf15fe80779"
        },
        "date": 1755597953134,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 489469,
            "range": "± 2272",
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
          "id": "6a48f11a8a6613966d8927bd4298909e25625cff",
          "message": "Merge pull request #255 from mripard/dependabot/cargo/bytemuck-1.23.2\n\nbuild(deps): bump bytemuck from 1.23.1 to 1.23.2",
          "timestamp": "2025-08-19T12:02:24+02:00",
          "tree_id": "b37b11f1d969623222faa4a9b956da9e93c4e362",
          "url": "https://github.com/mripard/dradis/commit/6a48f11a8a6613966d8927bd4298909e25625cff"
        },
        "date": 1755597969003,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 491283,
            "range": "± 2420",
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
          "id": "ae50e56d47686283b15a9cc6bcc9b5ce318683ed",
          "message": "Merge pull request #259 from mripard/dependabot/cargo/anyhow-1.0.99\n\nbuild(deps): bump anyhow from 1.0.98 to 1.0.99",
          "timestamp": "2025-08-19T12:24:59+02:00",
          "tree_id": "ff2dd10383788cdc7d77005e4302d522dce738c2",
          "url": "https://github.com/mripard/dradis/commit/ae50e56d47686283b15a9cc6bcc9b5ce318683ed"
        },
        "date": 1755599317322,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 487595,
            "range": "± 2656",
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
          "id": "a07d0ec21f0eec0d3c22d00b955f77b2e16c0af4",
          "message": "Merge pull request #257 from mripard/dependabot/cargo/bitflags-2.9.2\n\nbuild(deps): bump bitflags from 2.9.1 to 2.9.2",
          "timestamp": "2025-08-19T12:24:46+02:00",
          "tree_id": "b34abca32b2d96a6313d3c304f759b14f6b37628",
          "url": "https://github.com/mripard/dradis/commit/a07d0ec21f0eec0d3c22d00b955f77b2e16c0af4"
        },
        "date": 1755599319502,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 492664,
            "range": "± 2756",
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
          "id": "99bbcedf6b205b04588ada3d2178c90dede20d07",
          "message": "Merge pull request #258 from mripard/dependabot/cargo/thiserror-2.0.15\n\nbuild(deps): bump thiserror from 2.0.12 to 2.0.15",
          "timestamp": "2025-08-19T12:25:10+02:00",
          "tree_id": "7e157979af8afe270cf9f7262f3bc65787511382",
          "url": "https://github.com/mripard/dradis/commit/99bbcedf6b205b04588ada3d2178c90dede20d07"
        },
        "date": 1755599333869,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 496396,
            "range": "± 3138",
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
          "id": "56bdbcab826ca40c73bffff7d0bca56b27bbf0fd",
          "message": "Merge pull request #256 from mripard/dependabot/cargo/clap-4.5.45\n\nbuild(deps): bump clap from 4.5.42 to 4.5.45",
          "timestamp": "2025-08-19T12:25:21+02:00",
          "tree_id": "4b325a8889816df3fa8495b7ae0e52fb42f8dfa3",
          "url": "https://github.com/mripard/dradis/commit/56bdbcab826ca40c73bffff7d0bca56b27bbf0fd"
        },
        "date": 1755599347321,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 488470,
            "range": "± 2182",
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
            "email": "mripard@kernel.org",
            "name": "Maxime Ripard",
            "username": "mripard"
          },
          "distinct": true,
          "id": "c605d2091dee77130a70bb4eed9425b1ea21559e",
          "message": "Update facet\n\nSigned-off-by: Maxime Ripard <mripard@kernel.org>",
          "timestamp": "2025-08-19T13:42:41+02:00",
          "tree_id": "f986778c21a18cb01ec60f8dbcd56e93a3b229c3",
          "url": "https://github.com/mripard/dradis/commit/c605d2091dee77130a70bb4eed9425b1ea21559e"
        },
        "date": 1755604000320,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 493335,
            "range": "± 3044",
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
          "id": "47784096bcfb9cf67c77c90f58db1234eac69fdc",
          "message": "Merge pull request #254 from mripard/dependabot/cargo/redid-16d3ed3\n\nbuild(deps): bump redid from `862de87` to `16d3ed3`",
          "timestamp": "2025-08-19T14:08:06+02:00",
          "tree_id": "4ae1f6f376f0715abcca02c7e579c34d02a7809e",
          "url": "https://github.com/mripard/dradis/commit/47784096bcfb9cf67c77c90f58db1234eac69fdc"
        },
        "date": 1755605543621,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 493143,
            "range": "± 3663",
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
          "id": "6e9e423074e733c5a254f6ae868e633e03fc29ec",
          "message": "Merge pull request #260 from mripard/github-ci-fmt-fix\n\nTry to fix rustfmt (again)",
          "timestamp": "2025-08-19T14:50:07+02:00",
          "tree_id": "186649f79f19f00d0cf9ca0986076fb7285df2d5",
          "url": "https://github.com/mripard/dradis/commit/6e9e423074e733c5a254f6ae868e633e03fc29ec"
        },
        "date": 1755608033728,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 487627,
            "range": "± 2273",
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
          "id": "5780e2bab72e660e34c9dbe459418e089fa0a4aa",
          "message": "Merge pull request #274 from JoseExposito/bump-nucleid\n\ncargo: Update nucleid",
          "timestamp": "2025-09-10T13:51:21+02:00",
          "tree_id": "27445227c07ace85dc201009d37a3046e9be534c",
          "url": "https://github.com/mripard/dradis/commit/5780e2bab72e660e34c9dbe459418e089fa0a4aa"
        },
        "date": 1757505310895,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 487912,
            "range": "± 2532",
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
          "id": "b634775fbfa62cfd93c2d7c9080ce0af11f4064a",
          "message": "Merge pull request #238 from mripard/vimc-ci\n\nMediaController Improvements",
          "timestamp": "2025-09-15T09:49:03+02:00",
          "tree_id": "a5643fbd0d05803130a4a1690a04bff142528ad1",
          "url": "https://github.com/mripard/dradis/commit/b634775fbfa62cfd93c2d7c9080ce0af11f4064a"
        },
        "date": 1757922860988,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 492962,
            "range": "± 2993",
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
          "id": "d996b6f194e998a32b2cfe82e845daaaed722321",
          "message": "Merge pull request #275 from mripard/boomer-hotplug\n\nHandle hotplug events in boomer",
          "timestamp": "2025-09-19T18:52:47+02:00",
          "tree_id": "ce2224fc59b8fe0657ebc6319b65b131a17e81db",
          "url": "https://github.com/mripard/dradis/commit/d996b6f194e998a32b2cfe82e845daaaed722321"
        },
        "date": 1758301013438,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 482130,
            "range": "± 1239",
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
          "id": "ca65365f1577a51d4c8f66c2b96f28ad2f9653f6",
          "message": "Merge pull request #276 from mripard/cleanup-v4lise\n\nEnable common lints on v4lise",
          "timestamp": "2025-09-24T16:25:42+02:00",
          "tree_id": "bc98912cd59701411dc0c54ac4b30d694917a628",
          "url": "https://github.com/mripard/dradis/commit/ca65365f1577a51d4c8f66c2b96f28ad2f9653f6"
        },
        "date": 1758724201717,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 477664,
            "range": "± 1945",
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
          "id": "987a7257a9a1a5af50c09afb4ee1e07c591b593a",
          "message": "Merge pull request #277 from mripard/rust-1.90-lints\n\nRust 1.90 lints",
          "timestamp": "2025-09-25T11:38:44+02:00",
          "tree_id": "2d301a697fcea985ea603f14d117c15a8523366a",
          "url": "https://github.com/mripard/dradis/commit/987a7257a9a1a5af50c09afb4ee1e07c591b593a"
        },
        "date": 1758793384763,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 487089,
            "range": "± 1992",
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
          "id": "bef62dc2110a6374eed5abd74da8275733ae2a6e",
          "message": "Merge pull request #278 from mripard/cleanup-dradis\n\nEnable common lints in dradis",
          "timestamp": "2025-09-25T11:39:08+02:00",
          "tree_id": "cf759008ffb7e163997986119d15f45709774f28",
          "url": "https://github.com/mripard/dradis/commit/bef62dc2110a6374eed5abd74da8275733ae2a6e"
        },
        "date": 1758793410581,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 496298,
            "range": "± 3240",
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
          "id": "1f9fef61a69bf2a86e83e00921f45a4ed50809af",
          "message": "Merge pull request #273 from JoseExposito/bug-multiple-connectors\n\nboomer: Exit if multiple connector are available",
          "timestamp": "2025-10-07T17:10:38+02:00",
          "tree_id": "af14b141c5473bcb2c425b49311bca6c54a5ae94",
          "url": "https://github.com/mripard/dradis/commit/1f9fef61a69bf2a86e83e00921f45a4ed50809af"
        },
        "date": 1759850179013,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 499385,
            "range": "± 2712",
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
          "id": "c8b4dff78d59baf329529e529f18cd50620b8f89",
          "message": "Merge pull request #285 from mripard/more-tests-benchmarks\n\nBenchmarks and tests improvements",
          "timestamp": "2025-10-16T15:33:31+02:00",
          "tree_id": "908d12636a8e2ff172ea9fd867e115b823da8876",
          "url": "https://github.com/mripard/dradis/commit/c8b4dff78d59baf329529e529f18cd50620b8f89"
        },
        "date": 1760621882572,
        "tool": "cargo",
        "benches": [
          {
            "name": "decode_and_check_frame/xxhash2/valid",
            "value": 490671,
            "range": "± 3272",
            "unit": "ns/iter"
          },
          {
            "name": "decode_and_check_frame/xxhash2/swapped",
            "value": 1229628,
            "range": "± 4776",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}