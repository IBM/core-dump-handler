#!/bin/bash

export cmd=""$1



if [ "$cmd" = "pods" ]
then
    echo '{
  "items": [
    {
      "id": "51cd8bdaa13a65518e790d307359d33f9288fc82664879c609029b1a83862db6",
      "metadata": {
        "name": "crashing-app-699c49b4ff-86wrh",
        "uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086",
        "namespace": "default",
        "attempt": 0
      },
      "state": "SANDBOX_READY",
      "createdAt": "1618746959894040481",
      "labels": {
        "app": "crashing-app",
        "io.kubernetes.pod.name": "crashing-app-699c49b4ff-86wrh",
        "io.kubernetes.pod.namespace": "default",
        "io.kubernetes.pod.uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086",
        "pod-template-hash": "848dc79df4"
      },
      "annotations": {
        "kubernetes.io/config.seen": "2021-04-18T11:55:58.909472224Z",
        "kubernetes.io/config.source": "api",
        "kubernetes.io/psp": "ibm-privileged-psp"
      },
      "runtimeHandler": ""
    }
  ]
}'
fi

if [ "$cmd" = "inspectp" ]
then
    echo '{
  "status": {
    "id": "f7ca3e453aaf4b6a313f3047d5089ec3b2a14c64333f171f2b3bfed801f29665",
    "metadata": {
      "attempt": 0,
      "name": "crashing-app-699c49b4ff-86wrh",
      "namespace": "default",
      "uid": "1fc8b82e-5be7-43f0-a63f-2d8db75e90a9"
    },
    "state": "SANDBOX_READY",
    "createdAt": "2020-04-12T02:01:28.777032433Z",
    "network": {
      "additionalIps": [],
      "ip": "172.30.72.83"
    },
    "linux": {
      "namespaces": {
        "options": {
          "ipc": "POD",
          "network": "POD",
          "pid": "CONTAINER"
        }
      }
    },
    "labels": {
      "app": "crashing-app",
      "io.kubernetes.pod.name": "crashing-app-699c49b4ff-86wrh",
      "io.kubernetes.pod.namespace": "default",
      "io.kubernetes.pod.uid": "1fc8b82e-5be7-43f0-a63f-2d8db75e90a9",
      "pod-template-hash": "699c49b4ff"
    },
    "annotations": {
      "kubernetes.io/config.seen": "2020-04-12T02:01:28.154668879Z",
      "kubernetes.io/config.source": "api",
      "kubernetes.io/psp": "ibm-privileged-psp"
    },
    "runtimeHandler": ""
  },
  "info": {
    "pid": 14017,
    "processStatus": "running",
    "netNamespaceClosed": false,
    "image": "registry.eu-de.bluemix.net/armada-master/pause:3.1",
    "snapshotKey": "f7ca3e453aaf4b6a313f3047d5089ec3b2a14c64333f171f2b3bfed801f29665",
    "snapshotter": "overlayfs",
    "runtimeHandler": "",
    "runtimeType": "io.containerd.runc.v2",
    "runtimeOptions": {},
    "config": {
      "metadata": {
        "name": "crashing-app-699c49b4ff-86wrh",
        "uid": "1fc8b82e-5be7-43f0-a63f-2d8db75e90a9",
        "namespace": "default"
      },
      "hostname": "crashing-app-699c49b4ff-86wrh",
      "log_directory": "/var/log/pods/default_crashing-app-699c49b4ff-86wrh_1fc8b82e-5be7-43f0-a63f-2d8db75e90a9",
      "dns_config": {
        "servers": [
          "172.21.0.10"
        ],
        "searches": [
          "default.svc.cluster.local",
          "svc.cluster.local",
          "cluster.local"
        ],
        "options": [
          "ndots:5"
        ]
      },
      "labels": {
        "app": "crashing-app",
        "io.kubernetes.pod.name": "crashing-app-699c49b4ff-86wrh",
        "io.kubernetes.pod.namespace": "default",
        "io.kubernetes.pod.uid": "1fc8b82e-5be7-43f0-a63f-2d8db75e90a9",
        "pod-template-hash": "699c49b4ff"
      },
      "annotations": {
        "kubernetes.io/config.seen": "2020-04-12T02:01:28.154668879Z",
        "kubernetes.io/config.source": "api",
        "kubernetes.io/psp": "ibm-privileged-psp"
      },
      "linux": {
        "cgroup_parent": "/kubepods/besteffort/pod1fc8b82e-5be7-43f0-a63f-2d8db75e90a9",
        "security_context": {
          "namespace_options": {
            "pid": 1
          }
        }
      }
    },
    "runtimeSpec": {
      "ociVersion": "1.0.1-dev",
      "process": {
        "user": {
          "uid": 0,
          "gid": 0
        },
        "args": [
          "/pause"
        ],
        "env": [
          "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
        ],
        "cwd": "/",
        "capabilities": {
          "bounding": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_MKNOD",
            "CAP_NET_RAW",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETFCAP",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_SYS_CHROOT",
            "CAP_KILL",
            "CAP_AUDIT_WRITE"
          ],
          "effective": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_MKNOD",
            "CAP_NET_RAW",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETFCAP",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_SYS_CHROOT",
            "CAP_KILL",
            "CAP_AUDIT_WRITE"
          ],
          "inheritable": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_MKNOD",
            "CAP_NET_RAW",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETFCAP",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_SYS_CHROOT",
            "CAP_KILL",
            "CAP_AUDIT_WRITE"
          ],
          "permitted": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_MKNOD",
            "CAP_NET_RAW",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETFCAP",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_SYS_CHROOT",
            "CAP_KILL",
            "CAP_AUDIT_WRITE"
          ]
        },
        "noNewPrivileges": true,
        "oomScoreAdj": -998
      },
      "root": {
        "path": "rootfs",
        "readonly": true
      },
      "hostname": "crashing-app-699c49b4ff-86wrh",
      "mounts": [
        {
          "destination": "/proc",
          "type": "proc",
          "source": "proc",
          "options": [
            "nosuid",
            "noexec",
            "nodev"
          ]
        },
        {
          "destination": "/dev",
          "type": "tmpfs",
          "source": "tmpfs",
          "options": [
            "nosuid",
            "strictatime",
            "mode=755",
            "size=65536k"
          ]
        },
        {
          "destination": "/dev/pts",
          "type": "devpts",
          "source": "devpts",
          "options": [
            "nosuid",
            "noexec",
            "newinstance",
            "ptmxmode=0666",
            "mode=0620",
            "gid=5"
          ]
        },
        {
          "destination": "/dev/shm",
          "type": "tmpfs",
          "source": "shm",
          "options": [
            "nosuid",
            "noexec",
            "nodev",
            "mode=1777",
            "size=65536k"
          ]
        },
        {
          "destination": "/dev/mqueue",
          "type": "mqueue",
          "source": "mqueue",
          "options": [
            "nosuid",
            "noexec",
            "nodev"
          ]
        },
        {
          "destination": "/sys",
          "type": "sysfs",
          "source": "sysfs",
          "options": [
            "nosuid",
            "noexec",
            "nodev",
            "ro"
          ]
        },
        {
          "destination": "/dev/shm",
          "type": "bind",
          "source": "/run/containerd/io.containerd.grpc.v1.cri/sandboxes/f7ca3e453aaf4b6a313f3047d5089ec3b2a14c64333f171f2b3bfed801f29665/shm",
          "options": [
            "rbind",
            "ro"
          ]
        }
      ],
      "annotations": {
        "io.kubernetes.cri.container-type": "sandbox",
        "io.kubernetes.cri.sandbox-id": "f7ca3e453aaf4b6a313f3047d5089ec3b2a14c64333f171f2b3bfed801f29665",
        "io.kubernetes.cri.sandbox-log-directory": "/var/log/pods/default_crashing-app-699c49b4ff-86wrh_1fc8b82e-5be7-43f0-a63f-2d8db75e90a9"
      },
      "linux": {
        "resources": {
          "devices": [
            {
              "allow": false,
              "access": "rwm"
            }
          ],
          "cpu": {
            "shares": 2
          }
        },
        "cgroupsPath": "/kubepods/besteffort/pod1fc8b82e-5be7-43f0-a63f-2d8db75e90a9/f7ca3e453aaf4b6a313f3047d5089ec3b2a14c64333f171f2b3bfed801f29665",
        "namespaces": [
          {
            "type": "pid"
          },
          {
            "type": "ipc"
          },
          {
            "type": "uts"
          },
          {
            "type": "mount"
          },
          {
            "type": "network",
            "path": "/var/run/netns/cni-f6253b67-2766-fcf2-9100-439a32ce7a9b"
          }
        ],
        "maskedPaths": [
          "/proc/acpi",
          "/proc/asound",
          "/proc/kcore",
          "/proc/keys",
          "/proc/latency_stats",
          "/proc/timer_list",
          "/proc/timer_stats",
          "/proc/sched_debug",
          "/sys/firmware",
          "/proc/scsi"
        ],
        "readonlyPaths": [
          "/proc/bus",
          "/proc/fs",
          "/proc/irq",
          "/proc/sys",
          "/proc/sysrq-trigger"
        ]
      }
    },
    "cniResult": {
      "Interfaces": {
        "cali92b1ab6243b": {
          "IPConfigs": null,
          "Mac": "",
          "Sandbox": ""
        },
        "eth0": {
          "IPConfigs": [
            {
              "IP": "172.30.72.83",
              "Gateway": ""
            }
          ],
          "Mac": "",
          "Sandbox": ""
        }
      },
      "DNS": [
        {},
        {}
      ],
      "Routes": null
    }
  }
}'
fi

if [ "$cmd" = "ps" ]
then
    echo '{
  "containers": [
    {
      "id": "4bd48d7c6a03cd94a0e95e97011ed5d2ca72045723a5ed55da06fd54eff32b0a",
      "podSandboxId": "51cd8bdaa13a65518e790d307359d33f9288fc82664879c609029b1a83862db6",
      "metadata": {
        "name": "example-crashing-nodejs-app",
        "attempt": 7
      },
      "image": {
        "image": "sha256:3b8adc6c30f4e7e4afb57daef9d1c8af783a4a647a4670780e9df085c0525efa"
      },
      "imageRef": "sha256:3b8adc6c30f4e7e4afb57daef9d1c8af783a4a647a4670780e9df085c0525efa",
      "state": "CONTAINER_RUNNING",
      "createdAt": "1619258836379736566",
      "labels": {
        "io.kubernetes.container.name": "example-crashing-nodejs-app",
        "io.kubernetes.pod.name": "crashing-app-699c49b4ff-86wrh",
        "io.kubernetes.pod.namespace": "default",
        "io.kubernetes.pod.uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086"
      },
      "annotations": {
        "io.kubernetes.container.hash": "992bb403",
        "io.kubernetes.container.restartCount": "7",
        "io.kubernetes.container.terminationMessagePath": "/dev/termination-log",
        "io.kubernetes.container.terminationMessagePolicy": "File",
        "io.kubernetes.pod.terminationGracePeriod": "30"
      }
    }
  ]
}'
fi 

if [ "$cmd" = "img" ]
then
    echo '{
  "images": [
    {
      "id": "sha256:e7b300aee9f9bf3433d32bc9305bfdd22183beb59d933b48d77ab56ba53a197a",
      "repoTags": [
        "docker.io/library/alpine:3.10"
      ],
      "repoDigests": [
        "docker.io/library/alpine@sha256:451eee8bedcb2f029756dc3e9d73bab0e7943c1ac55cff3a4861c52a0fdd3e98"
      ],
      "size": "2801976",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:388056c9a6838deea3792e8f00705b35b439cf57b3c9c2634fb4e95cfc896de6",
      "repoTags": [
        "docker.io/library/busybox:latest"
      ],
      "repoDigests": [
        "docker.io/library/busybox@sha256:ae39a6f5c07297d7ab64dbd4f82c77c874cc6a94cea29fdec309d0992574b4f7"
      ],
      "size": "768773",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:3b8adc6c30f4e7e4afb57daef9d1c8af783a4a647a4670780e9df085c0525efa",
      "repoTags": [
        "docker.io/number9/example-crashing-nodejs-app:latest"
      ],
      "repoDigests": [
        "docker.io/number9/example-crashing-nodejs-app@sha256:b8fea40ed9da77307702608d1602a812c5983e0ec0b788fc6298985a40be3800"
      ],
      "size": "338054458",
      "uid": null,
      "username": "node"
    },
    {
      "id": "sha256:e55fe45170374f40f0eb76491f3fd3f638e1307e641bf197b0168f712ae414b1",
      "repoTags": [
        "docker.io/number9/kcdt:v1.2.8"
      ],
      "repoDigests": [
        "docker.io/number9/kcdt@sha256:923dab90191e870534ca5fe3f0948d2182cd33b4fd5fb690247ed510a198bfec"
      ],
      "size": "26710628",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:4b97dc265156e2bc2fb2567003489cbf2b7d1e538e6b15712a12668d6aaa00fd",
      "repoTags": [
        "icr.io/ibm/ibmcloud-object-storage-driver:1.8.16"
      ],
      "repoDigests": [
        "icr.io/ibm/ibmcloud-object-storage-driver@sha256:c796a4c693b4b7bf366c89208e96648d082836ebcb3bd03d8b63aca6883a69b0"
      ],
      "size": "103453889",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:22a5079aa3d8b81f65ab14ef106f81ac768f96356bb19cb9edbc8a777682afe3",
      "repoTags": [
        "icr.io/ibm/ibmcloud-object-storage-plugin:1.8.16"
      ],
      "repoDigests": [
        "icr.io/ibm/ibmcloud-object-storage-plugin@sha256:9c73804b37a3272dc42073a16bee014f33d7d322afe8061be5af8d8c3d72de89"
      ],
      "size": "102609165",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:b7db21b30ad90d631a5c1d8820146ccecc31b6dfb7a8ff556b7edafed218be88",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/addon-resizer:1.8.11"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/addon-resizer@sha256:35745de3c9a2884d53ad0e81b39f1eed9a7c77f5f909b9e84f9712b37ffb3021"
      ],
      "size": "9347950",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:4ced78f12570461f38f90d7b095da91259fe2b6d1ea9eb8a68c9f22e33808b14",
      "repoTags": [
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/alpine@sha256:7cabdf4563795f652c71497a0399c68edfda2a0627333fe984faa0c68c5188c6"
      ],
      "size": "4965159",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:f263c183e5db2982dca3039ec1786282facb76ccb15b27e5a338c456fcb4d162",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/alpine:3.13.1"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/alpine@sha256:b3779a83448f615da1c2379c405027ed48d05ace1334bfc12866507c49a64a71"
      ],
      "size": "5087430",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:7940746221f3d5efbba0a813ee81696dc31af32276021a45c1e1e089fe464ff6",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/armada-calico-extension:618"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/armada-calico-extension@sha256:bbb377c7bec633911416847787a6634c06297db77b50359f70c8e9f86ee96fe8"
      ],
      "size": "67622833",
      "uid": {
        "value": "2000"
      },
      "username": ""
    },
    {
      "id": "sha256:da995a8de478db8c200f98058b5e5be3ba1b7d73bc6e6724e86633f46867493e",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/calico/cni:v3.16.8"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/calico/cni@sha256:d815d497b31d871d33a7cb84516c33ee7f47141522682ee3f89b559be96a6c64"
      ],
      "size": "48347935",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:dbc755668c26ff517625f53d6c8d7ec8055336ba9f603483eabecbd0e4f9afce",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/calico/kube-controllers:v3.16.8"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/calico/kube-controllers@sha256:6141de9bfb6e0270e680acc54d5ec365380e3788188e7e0ae03ecf5cfb6a79b5"
      ],
      "size": "23095776",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:dca8b0edd3762d817c03308ba46ae9d9d57c970285d4295309e6b970098f7735",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/calico/node:v3.16.8"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/calico/node@sha256:20d4e415eaa3b035ebafa26a59d5bcd8b24ef6ba53cbc0819aa7c32ea478d452"
      ],
      "size": "62311209",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:da14e8a080575b36083841de483ade188a0d01745048b6723e25c73ad7454ddc",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/calico/typha:v3.16.8"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/calico/typha@sha256:aec860283583e533406254a1e285344ed2245eef19d205087c18f6904cc67b5b"
      ],
      "size": "22529588",
      "uid": {
        "value": "999"
      },
      "username": ""
    },
    {
      "id": "sha256:078b6f04135ffa227c125f8b7cb1f681df498bfa3212b46c457972801edcc648",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/cluster-proportional-autoscaler-amd64:1.8.3"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/cluster-proportional-autoscaler-amd64@sha256:dce43068853ad396b0fb5ace9a56cc14114e31979e241342d12d04526be1dfcc"
      ],
      "size": "15190383",
      "uid": null,
      "username": "nonroot"
    },
    {
      "id": "sha256:369e6326a8836a73accbb49bb281c2f3820e813db155eeeb857598fc9a583662",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/configmap-operator-registry:v1.15.3"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/configmap-operator-registry@sha256:ca3b98cce6a117d3a828e5f519e5137b7a382a1d053cc726f7bf0db3dedb769d"
      ],
      "size": "35857476",
      "uid": {
        "value": "1001"
      },
      "username": ""
    },
    {
      "id": "sha256:296a6d5035e2d6919249e02709a488d680ddca91357602bd65e605eac967b899",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/coredns:1.8.0"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/coredns@sha256:10ecc12177735e5a6fd6fa0127202776128d860ed7ab0341780ddaeb1f6dfe61"
      ],
      "size": "12943490",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:9355d058680c4f3a0c6080897ae59c293fd0e276fb4e19abef8b1137cfb099d7",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/haproxy:9b2dca4a105f435722cf829217ee4612ff069a49"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/haproxy@sha256:694b38a6ff83224371f55cefeab65300cc1382fa87f8259a1a600d8f13d70a62"
      ],
      "size": "82818598",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:d3cfbff43993ddd18a86f5e948413b7d7c9467656805369d233776724e8f48a5",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/ingress-alpine:3.13"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/ingress-alpine@sha256:5eff3cd80b71cdd3edc1d9b644d235053ce8430b1f4b689b5d131193635421ff"
      ],
      "size": "3560897",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:6a7f2a09fb674ddf45d1fe74d7415d145065dfa3f199a753f44e379ebe9780e0",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community:0.35.0_1094_iks"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community@sha256:d8ab4d2cd9255c4419b49448961da8141d8825b06291481d9ed291bd0e24af09"
      ],
      "size": "122012788",
      "uid": null,
      "username": "www-data"
    },
    {
      "id": "sha256:7e432fa06d04a2fedef23df59113e929c34ac5bb1c7dcf1ed0870c13bc71ee0a",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community:0.35.0_1155_iks"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community@sha256:1d7122b6c504a2e2146ee0467115c9f1ced91de622df9a39aa13c86cfeb1aa11"
      ],
      "size": "122670008",
      "uid": null,
      "username": "www-data"
    },
    {
      "id": "sha256:e16b6e9d19cb9ddc8b6bfb255a3827867e8a31dad3001758d441a6d164f49d60",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community:0.35.0_1182_iks"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community@sha256:b276ea7be467da8dd0b08920d75bb7082f2a4391ac601f5b6e94db0ffa64164b"
      ],
      "size": "122728981",
      "uid": null,
      "username": "www-data"
    },
    {
      "id": "sha256:358730fbf0c697011f4c08f20a2cbafec9d9e79b3f1639dd9290e64186dab7bf",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community:0.45.0_1228_iks"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/ingress-community@sha256:94eb77e28ff922d0bfbc2f8df3f213827c95110aae970e64d318859733077140"
      ],
      "size": "109983177",
      "uid": null,
      "username": "www-data"
    },
    {
      "id": "sha256:dbbf966b81b7deaf53c517b2344efc63b71ce09497d74680c6a79b35b641d038",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/keepalived-watcher:1274"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/keepalived-watcher@sha256:400da98afc91eb8c33761059e4a4e57adb2a468fe9de1930558d3127af89b784"
      ],
      "size": "14703308",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:1b9aee522069390cb84c17af636dc543d76684bae3a5b23397976cf936bb56b0",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/keepalived:1274"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/keepalived@sha256:1134d6840cd5c82858d9ccb9290d5944579e031fb0167487da8c770d8136d843"
      ],
      "size": "15653318",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:fd110d63b15bd3bd37f0815741b06110e4c151eb7118b70c6450abb1a436ffc4",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/kubernetesui-dashboard:v2.0.5"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/kubernetesui-dashboard@sha256:9abf71e50b3a6fb644452d49eb738278f43786f3a21c48bfba3fd831faa83512"
      ],
      "size": "71274933",
      "uid": null,
      "username": "nonroot"
    },
    {
      "id": "sha256:48d79e554db69811a12d0300d8ad5da158d134d575d8268902430d824143eb49",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/kubernetesui-metrics-scraper:v1.0.6"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/kubernetesui-metrics-scraper@sha256:328547d4f7d729ff1178cff9bc23d82801392e60b183b240755a2216caa18df6"
      ],
      "size": "15544447",
      "uid": null,
      "username": "nonroot"
    },
    {
      "id": "sha256:07c9e703ca2c3a37741cecadd7ea8dd7182c3381ceebd7631413824f0f62ed09",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/metrics-server:v0.3.7"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/metrics-server@sha256:c0efe772bb9e5c289db6cc4bc2002c268507d0226f2a3815f7213e00261c38e9"
      ],
      "size": "21031646",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:f8deeefc311a6a9fe853c7a57337ac632bdad1be4a7c71cfb9266aa371da550c",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/olm:0.16.1-IKS-5"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/olm@sha256:1c6c2393a823653e3c7c92bdc6af118ffbace23f27ea6dd1c41befbe182ee053"
      ],
      "size": "78705635",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:80d28bedfe5dec59da9ebf8e6260224ac9008ab5c11dbbe16ee3ba3e4439ac2c",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/pause:3.2"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/pause@sha256:4a1c4b21597c1b4415bdbecb28a3296c6b5e23ca4f9feeb599860a1dac6a0108"
      ],
      "size": "297819",
      "uid": null,
      "username": ""
    },
    {
      "id": "sha256:d08660f3c1b4ce000dd7b122a3974a3e31b91116b0bb26449c0f38788c69bf9d",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/storage-file-plugin:389"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/storage-file-plugin@sha256:a256fce01c26228a37a7f7d09af1f42f97c6a974dbbc0cd55a73e99cc875e95e"
      ],
      "size": "267614172",
      "uid": {
        "value": "2000"
      },
      "username": ""
    },
    {
      "id": "sha256:0346f8bd4c0d32f873fbac2716dc4f09f7a6ea70ce9d9d5d04ce71fcd9a07ef1",
      "repoTags": [
        "registry.eu-de.bluemix.net/armada-master/vpn-client:2.4.6-r3-IKS-301"
      ],
      "repoDigests": [
        "registry.eu-de.bluemix.net/armada-master/vpn-client@sha256:eadf26e519faf3bd8c156d567430d265e084f1395f6bf68eddec5640e38281f6"
      ],
      "size": "3830890",
      "uid": null,
      "username": ""
    }
  ]
}'
fi

if [ "$cmd" = "logs" ]
then
echo 'A LOG'
fi