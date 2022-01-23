#!/bin/bash

export cmd=""$1



if [ "$cmd" = "pods" ]
then
    echo '{
    "items": [
        {
            "annotations": {
                "kubernetes.io/config.seen": "2022-01-07T11:33:26.146624712-06:00",
                "kubernetes.io/config.source": "api"
            },
            "createdAt": "1641576806500927930",
            "id": "134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50",
            "labels": {
                "io.kubernetes.container.name": "POD",
                "io.kubernetes.pod.name": "segfaulter",
                "io.kubernetes.pod.namespace": "default",
                "io.kubernetes.pod.uid": "aaaa2b4a-f398-41c0-928b-049e1cc4ec40",
                "run": "segfaulter"
            },
            "metadata": {
                "attempt": 0,
                "name": "segfaulter",
                "namespace": "default",
                "uid": "aaaa2b4a-f398-41c0-928b-049e1cc4ec40"
            },
            "runtimeHandler": "",
            "state": "SANDBOX_READY"
        },
        {
            "annotations": {
                "kubernetes.io/config.seen": "2022-01-07T11:28:49.184017050-06:00",
                "kubernetes.io/config.source": "api"
            },
            "createdAt": "1641576529587618605",
            "id": "c8bd7c0f3406c10e0c426118abba864f92f05dbe24c829d32bf09ccae15386df",
            "labels": {
                "io.kubernetes.container.name": "POD",
                "io.kubernetes.pod.name": "segfaulter",
                "io.kubernetes.pod.namespace": "default",
                "io.kubernetes.pod.uid": "1aff5750-5cd8-46cd-b572-bdb3b4a81a2a",
                "run": "segfaulter"
            },
            "metadata": {
                "attempt": 0,
                "name": "segfaulter",
                "namespace": "default",
                "uid": "1aff5750-5cd8-46cd-b572-bdb3b4a81a2a"
            },
            "runtimeHandler": "",
            "state": "SANDBOX_NOTREADY"
        }
    ]
}'
fi

if [ "$cmd" = "inspectp" ]
then
    echo '{
  "status": {
    "id": "134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50",
    "metadata": {
      "attempt": 0,
      "name": "segfaulter",
      "namespace": "default",
      "uid": "aaaa2b4a-f398-41c0-928b-049e1cc4ec40"
    },
    "state": "SANDBOX_READY",
    "createdAt": "2022-01-07T11:33:26.50092793-06:00",
    "network": {
      "additionalIps": [],
      "ip": "172.30.129.95"
    },
    "linux": {
      "namespaces": {
        "options": {
          "ipc": "POD",
          "network": "POD",
          "pid": "CONTAINER",
          "targetId": ""
        }
      }
    },
    "labels": {
      "io.kubernetes.container.name": "POD",
      "io.kubernetes.pod.name": "segfaulter",
      "io.kubernetes.pod.namespace": "default",
      "io.kubernetes.pod.uid": "aaaa2b4a-f398-41c0-928b-049e1cc4ec40",
      "run": "segfaulter"
    },
    "annotations": {
      "kubernetes.io/config.seen": "2022-01-07T11:33:26.146624712-06:00",
      "kubernetes.io/config.source": "api"
    },
    "runtimeHandler": ""
  },
  "info": {
    "image": "registry.eu-gb.bluemix.net/armada-master/pause:3.5",
    "pid": 38091,
    "runtimeSpec": {
      "ociVersion": "1.0.2-dev",
      "process": {
        "user": {
          "uid": 0,
          "gid": 0
        },
        "args": [
          "/pause"
        ],
        "env": [
          "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
          "TERM=xterm"
        ],
        "cwd": "/",
        "capabilities": {
          "bounding": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_KILL"
          ],
          "effective": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_KILL"
          ],
          "inheritable": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_KILL"
          ],
          "permitted": [
            "CAP_CHOWN",
            "CAP_DAC_OVERRIDE",
            "CAP_FSETID",
            "CAP_FOWNER",
            "CAP_SETGID",
            "CAP_SETUID",
            "CAP_SETPCAP",
            "CAP_NET_BIND_SERVICE",
            "CAP_KILL"
          ]
        },
        "oomScoreAdj": -998,
        "selinuxLabel": "system_u:system_r:svirt_lxc_net_t:s0:c739,c982"
      },
      "root": {
        "path": "/var/data/criorootstorage/overlay/3130d8abc8e7df1f80bf42606dc8e1187622b068e349e3edd00e9c859cf765c7/merged",
        "readonly": true
      },
      "hostname": "segfaulter",
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
          "destination": "/etc/resolv.conf",
          "type": "bind",
          "source": "/var/data/crioruntimestorage/overlay-containers/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50/userdata/resolv.conf",
          "options": [
            "ro",
            "bind",
            "nodev",
            "nosuid",
            "noexec"
          ]
        },
        {
          "destination": "/dev/shm",
          "type": "bind",
          "source": "/var/data/crioruntimestorage/overlay-containers/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50/userdata/shm",
          "options": [
            "rw",
            "bind"
          ]
        },
        {
          "destination": "/etc/hostname",
          "type": "bind",
          "source": "/var/data/crioruntimestorage/overlay-containers/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50/userdata/hostname",
          "options": [
            "ro",
            "bind",
            "nodev",
            "nosuid",
            "noexec"
          ]
        }
      ],
      "annotations": {
        "io.kubernetes.cri-o.CgroupParent": "kubepods-besteffort-podaaaa2b4a_f398_41c0_928b_049e1cc4ec40.slice",
        "io.kubernetes.pod.name": "segfaulter",
        "run": "segfaulter",
        "io.kubernetes.cri-o.ContainerName": "k8s_POD_segfaulter_default_aaaa2b4a-f398-41c0-928b-049e1cc4ec40_0",
        "io.kubernetes.cri-o.PortMappings": "[]",
        "io.kubernetes.cri-o.HostNetwork": "false",
        "io.kubernetes.container.name": "POD",
        "io.kubernetes.cri-o.CNIResult": "{\"cniVersion\":\"0.4.0\",\"interfaces\":[{\"name\":\"calif12c8c41796\"}],\"ips\":[{\"version\":\"4\",\"address\":\"172.30.129.95/32\"}],\"dns\":{}}",
        "io.kubernetes.cri-o.SandboxID": "134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50",
        "io.kubernetes.cri-o.RuntimeHandler": "",
        "io.kubernetes.cri-o.HostName": "segfaulter",
        "io.kubernetes.cri-o.NamespaceOptions": "{\"pid\":1}",
        "io.container.manager": "cri-o",
        "org.systemd.property.CollectMode": "'inactive-or-failed'",
        "io.kubernetes.pod.uid": "aaaa2b4a-f398-41c0-928b-049e1cc4ec40",
        "io.kubernetes.cri-o.SeccompProfilePath": "runtime/default",
        "io.kubernetes.cri-o.Metadata": "{\"Name\":\"segfaulter\",\"UID\":\"aaaa2b4a-f398-41c0-928b-049e1cc4ec40\",\"Namespace\":\"default\",\"Attempt\":0}",
        "io.kubernetes.cri-o.Image": "registry.eu-gb.bluemix.net/armada-master/pause:3.5",
        "io.kubernetes.cri-o.MountPoint": "/var/data/criorootstorage/overlay/3130d8abc8e7df1f80bf42606dc8e1187622b068e349e3edd00e9c859cf765c7/merged",
        "io.kubernetes.cri-o.HostnamePath": "/var/data/crioruntimestorage/overlay-containers/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50/userdata/hostname",
        "io.kubernetes.cri-o.IP.0": "172.30.129.95",
        "io.kubernetes.cri-o.Annotations": "{\"kubernetes.io/config.source\":\"api\",\"kubernetes.io/config.seen\":\"2022-01-07T11:33:26.146624712-06:00\"}",
        "io.kubernetes.cri-o.Created": "2022-01-07T11:33:26.50092793-06:00",
        "kubernetes.io/config.seen": "2022-01-07T11:33:26.146624712-06:00",
        "io.kubernetes.cri-o.ContainerType": "sandbox",
        "io.kubernetes.cri-o.KubeName": "segfaulter",
        "io.kubernetes.cri-o.Namespace": "default",
        "io.kubernetes.cri-o.Labels": "{\"io.kubernetes.container.name\":\"POD\",\"io.kubernetes.pod.namespace\":\"default\",\"io.kubernetes.pod.name\":\"segfaulter\",\"run\":\"segfaulter\",\"io.kubernetes.pod.uid\":\"aaaa2b4a-f398-41c0-928b-049e1cc4ec40\"}",
        "io.kubernetes.cri-o.Name": "k8s_segfaulter_default_aaaa2b4a-f398-41c0-928b-049e1cc4ec40_0",
        "io.kubernetes.pod.namespace": "default",
        "io.kubernetes.cri-o.LogPath": "/var/log/pods/default_segfaulter_aaaa2b4a-f398-41c0-928b-049e1cc4ec40/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50.log",
        "io.kubernetes.cri-o.PrivilegedRuntime": "false",
        "io.kubernetes.cri-o.ResolvPath": "/var/data/crioruntimestorage/overlay-containers/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50/userdata/resolv.conf",
        "kubernetes.io/config.source": "api",
        "io.kubernetes.cri-o.ContainerID": "134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50",
        "io.kubernetes.cri-o.ShmPath": "/var/data/crioruntimestorage/overlay-containers/134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50/userdata/shm"
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
        "cgroupsPath": "kubepods-besteffort-podaaaa2b4a_f398_41c0_928b_049e1cc4ec40.slice:crio:134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50",
        "namespaces": [
          {
            "type": "pid"
          },
          {
            "type": "network",
            "path": "/var/run/netns/9abac4ae-94a6-4853-b299-e4c1c2c508c0"
          },
          {
            "type": "ipc",
            "path": "/var/run/ipcns/9abac4ae-94a6-4853-b299-e4c1c2c508c0"
          },
          {
            "type": "uts",
            "path": "/var/run/utsns/9abac4ae-94a6-4853-b299-e4c1c2c508c0"
          },
          {
            "type": "mount"
          }
        ],
        "seccomp": {
          "defaultAction": "SCMP_ACT_ERRNO",
          "architectures": [
            "SCMP_ARCH_X86_64",
            "SCMP_ARCH_X86",
            "SCMP_ARCH_X32"
          ],
          "syscalls": [
            {
              "names": [
                "_llseek",
                "_newselect",
                "accept",
                "accept4",
                "access",
                "adjtimex",
                "alarm",
                "bind",
                "brk",
                "capget",
                "capset",
                "chdir",
                "chmod",
                "chown",
                "chown32",
                "clock_adjtime",
                "clock_adjtime64",
                "clock_getres",
                "clock_getres_time64",
                "clock_gettime",
                "clock_gettime64",
                "clock_nanosleep",
                "clock_nanosleep_time64",
                "clone",
                "close",
                "close_range",
                "connect",
                "copy_file_range",
                "creat",
                "dup",
                "dup2",
                "dup3",
                "epoll_create",
                "epoll_create1",
                "epoll_ctl",
                "epoll_ctl_old",
                "epoll_pwait",
                "epoll_pwait2",
                "epoll_wait",
                "epoll_wait_old",
                "eventfd",
                "eventfd2",
                "execve",
                "execveat",
                "exit",
                "exit_group",
                "faccessat",
                "faccessat2",
                "fadvise64",
                "fadvise64_64",
                "fallocate",
                "fanotify_mark",
                "fchdir",
                "fchmod",
                "fchmodat",
                "fchown",
                "fchown32",
                "fchownat",
                "fcntl",
                "fcntl64",
                "fdatasync",
                "fgetxattr",
                "flistxattr",
                "flock",
                "fork",
                "fremovexattr",
                "fsconfig",
                "fsetxattr",
                "fsmount",
                "fsopen",
                "fspick",
                "fstat",
                "fstat64",
                "fstatat64",
                "fstatfs",
                "fstatfs64",
                "fsync",
                "ftruncate",
                "ftruncate64",
                "futex",
                "futimesat",
                "get_robust_list",
                "get_thread_area",
                "getcpu",
                "getcwd",
                "getdents",
                "getdents64",
                "getegid",
                "getegid32",
                "geteuid",
                "geteuid32",
                "getgid",
                "getgid32",
                "getgroups",
                "getgroups32",
                "getitimer",
                "getpeername",
                "getpgid",
                "getpgrp",
                "getpid",
                "getppid",
                "getpriority",
                "getrandom",
                "getresgid",
                "getresgid32",
                "getresuid",
                "getresuid32",
                "getrlimit",
                "getrusage",
                "getsid",
                "getsockname",
                "getsockopt",
                "gettid",
                "gettimeofday",
                "getuid",
                "getuid32",
                "getxattr",
                "inotify_add_watch",
                "inotify_init",
                "inotify_init1",
                "inotify_rm_watch",
                "io_cancel",
                "io_destroy",
                "io_getevents",
                "io_setup",
                "io_submit",
                "ioctl",
                "ioprio_get",
                "ioprio_set",
                "ipc",
                "keyctl",
                "kill",
                "lchown",
                "lchown32",
                "lgetxattr",
                "link",
                "linkat",
                "listen",
                "listxattr",
                "llistxattr",
                "lremovexattr",
                "lseek",
                "lsetxattr",
                "lstat",
                "lstat64",
                "madvise",
                "memfd_create",
                "mincore",
                "mkdir",
                "mkdirat",
                "mknod",
                "mknodat",
                "mlock",
                "mlock2",
                "mlockall",
                "mmap",
                "mmap2",
                "mount",
                "move_mount",
                "mprotect",
                "mq_getsetattr",
                "mq_notify",
                "mq_open",
                "mq_timedreceive",
                "mq_timedsend",
                "mq_unlink",
                "mremap",
                "msgctl",
                "msgget",
                "msgrcv",
                "msgsnd",
                "msync",
                "munlock",
                "munlockall",
                "munmap",
                "name_to_handle_at",
                "nanosleep",
                "newfstatat",
                "open",
                "openat",
                "openat2",
                "open_tree",
                "pause",
                "pidfd_getfd",
                "pidfd_open",
                "pidfd_send_signal",
                "pipe",
                "pipe2",
                "pivot_root",
                "poll",
                "ppoll",
                "ppoll_time64",
                "prctl",
                "pread64",
                "preadv",
                "preadv2",
                "prlimit64",
                "pselect6",
                "pselect6_time64",
                "pwrite64",
                "pwritev",
                "pwritev2",
                "read",
                "readahead",
                "readlink",
                "readlinkat",
                "readv",
                "reboot",
                "recv",
                "recvfrom",
                "recvmmsg",
                "recvmsg",
                "remap_file_pages",
                "removexattr",
                "rename",
                "renameat",
                "renameat2",
                "restart_syscall",
                "rmdir",
                "rt_sigaction",
                "rt_sigpending",
                "rt_sigprocmask",
                "rt_sigqueueinfo",
                "rt_sigreturn",
                "rt_sigsuspend",
                "rt_sigtimedwait",
                "rt_tgsigqueueinfo",
                "sched_get_priority_max",
                "sched_get_priority_min",
                "sched_getaffinity",
                "sched_getattr",
                "sched_getparam",
                "sched_getscheduler",
                "sched_rr_get_interval",
                "sched_setaffinity",
                "sched_setattr",
                "sched_setparam",
                "sched_setscheduler",
                "sched_yield",
                "seccomp",
                "select",
                "semctl",
                "semget",
                "semop",
                "semtimedop",
                "send",
                "sendfile",
                "sendfile64",
                "sendmmsg",
                "sendmsg",
                "sendto",
                "set_robust_list",
                "set_thread_area",
                "set_tid_address",
                "setfsgid",
                "setfsgid32",
                "setfsuid",
                "setfsuid32",
                "setgid",
                "setgid32",
                "setgroups",
                "setgroups32",
                "setitimer",
                "setpgid",
                "setpriority",
                "setregid",
                "setregid32",
                "setresgid",
                "setresgid32",
                "setresuid",
                "setresuid32",
                "setreuid",
                "setreuid32",
                "setrlimit",
                "setsid",
                "setsockopt",
                "setuid",
                "setuid32",
                "setxattr",
                "shmat",
                "shmctl",
                "shmdt",
                "shmget",
                "shutdown",
                "sigaltstack",
                "signalfd",
                "signalfd4",
                "sigreturn",
                "socketcall",
                "socketpair",
                "splice",
                "stat",
                "stat64",
                "statfs",
                "statfs64",
                "statx",
                "symlink",
                "symlinkat",
                "sync",
                "sync_file_range",
                "syncfs",
                "sysinfo",
                "syslog",
                "tee",
                "tgkill",
                "time",
                "timer_create",
                "timer_delete",
                "timer_getoverrun",
                "timer_gettime",
                "timer_gettime64",
                "timer_settime",
                "timerfd_create",
                "timerfd_gettime",
                "timerfd_gettime64",
                "timerfd_settime",
                "timerfd_settime64",
                "times",
                "tkill",
                "truncate",
                "truncate64",
                "ugetrlimit",
                "umask",
                "umount",
                "umount2",
                "uname",
                "unlink",
                "unlinkat",
                "unshare",
                "utime",
                "utimensat",
                "utimensat_time64",
                "utimes",
                "vfork",
                "wait4",
                "waitid",
                "waitpid",
                "write",
                "writev"
              ],
              "action": "SCMP_ACT_ALLOW"
            },
            {
              "names": [
                "personality"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 0,
                  "value": 0,
                  "op": "SCMP_CMP_EQ"
                }
              ]
            },
            {
              "names": [
                "personality"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 0,
                  "value": 8,
                  "op": "SCMP_CMP_EQ"
                }
              ]
            },
            {
              "names": [
                "personality"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 0,
                  "value": 131072,
                  "op": "SCMP_CMP_EQ"
                }
              ]
            },
            {
              "names": [
                "personality"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 0,
                  "value": 131080,
                  "op": "SCMP_CMP_EQ"
                }
              ]
            },
            {
              "names": [
                "personality"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 0,
                  "value": 4294967295,
                  "op": "SCMP_CMP_EQ"
                }
              ]
            },
            {
              "names": [
                "arch_prctl"
              ],
              "action": "SCMP_ACT_ALLOW"
            },
            {
              "names": [
                "modify_ldt"
              ],
              "action": "SCMP_ACT_ALLOW"
            },
            {
              "names": [
                "socket"
              ],
              "action": "SCMP_ACT_ERRNO",
              "errnoRet": 22,
              "args": [
                {
                  "index": 0,
                  "value": 16,
                  "op": "SCMP_CMP_EQ"
                },
                {
                  "index": 2,
                  "value": 9,
                  "op": "SCMP_CMP_EQ"
                }
              ]
            },
            {
              "names": [
                "socket"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 2,
                  "value": 9,
                  "op": "SCMP_CMP_NE"
                }
              ]
            },
            {
              "names": [
                "socket"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 0,
                  "value": 16,
                  "op": "SCMP_CMP_NE"
                }
              ]
            },
            {
              "names": [
                "socket"
              ],
              "action": "SCMP_ACT_ALLOW",
              "args": [
                {
                  "index": 2,
                  "value": 9,
                  "op": "SCMP_CMP_NE"
                }
              ]
            }
          ]
        },
        "mountLabel": "system_u:object_r:svirt_sandbox_file_t:s0:c739,c982"
      }
    }
  }
}'
fi

if [ "$cmd" = "ps" ]
then
    echo '{
    "containers": [
        {
            "annotations": {
                "io.kubernetes.container.hash": "fc3b83b3",
                "io.kubernetes.container.restartCount": "0",
                "io.kubernetes.container.terminationMessagePath": "/dev/termination-log",
                "io.kubernetes.container.terminationMessagePolicy": "File",
                "io.kubernetes.pod.terminationGracePeriod": "30"
            },
            "createdAt": "1641576808817824971",
            "id": "0e04af54d9273f5bb37eddbe8ace750275d7939612dd4864c792168cce2cff82",
            "image": {
                "annotations": {},
                "image": "quay.io/icdh/segfaulter@sha256:0630afbcfebb45059794b9a9f160f57f50062d28351c49bb568a3f7e206855bd"
            },
            "imageRef": "quay.io/icdh/segfaulter@sha256:0630afbcfebb45059794b9a9f160f57f50062d28351c49bb568a3f7e206855bd",
            "labels": {
                "io.kubernetes.container.name": "segfaulter",
                "io.kubernetes.pod.name": "segfaulter",
                "io.kubernetes.pod.namespace": "default",
                "io.kubernetes.pod.uid": "aaaa2b4a-f398-41c0-928b-049e1cc4ec40"
            },
            "metadata": {
                "attempt": 0,
                "name": "segfaulter"
            },
            "podSandboxId": "134b58ab2e0cfd7432a9db818b1b4ec52fdc747333f0ba2c9342860dc2ea7c50",
            "state": "CONTAINER_RUNNING"
        }
    ]
}'
fi


if [ "$cmd" = "image" ]
then
    echo '{
  "images": [
    {
    "id": "d8087c58ebe51554d52054e955680805d86969dc9b6917f5e3fa3ecb81c86e33",
    "repoDigests": [
        "quay.io/icdh/segfaulter@sha256:0630afbcfebb45059794b9a9f160f57f50062d28351c49bb568a3f7e206855bd"
    ],
    "repoTags": [
        "quay.io/icdh/segfaulter:latest"
    ],
    "size": "10229047",
    "spec": null,
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