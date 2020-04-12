# IBM Core Dump Handler

This helm chart is designed to deploy functionality to automatically save core dumps from [IBM Kubernetes Service](https://cloud.ibm.com/docs/containers?topic=containers-getting-started) pods or [RedHat OpenShift Kubernetes Service](https://cloud.ibm.com/kubernetes/catalog/create?platformType=openshift) pods to [IBM Cloud Object Storage](https://cloud.ibm.com/docs/services/cloud-object-storage?topic=cloud-object-storage-about-ibm-cloud-object-storage#about-ibm-cloud-object-storage).


## Introduction

[Core Dumps](https://en.wikipedia.org/wiki/Core_dump) are a critical part of observability.

As systems become more distributed core dumps offer teams a non-invasive approach to understanding why programs are malfunctioning in any environment they are deployed to. 

Core Dumps are useful in a wide number of scenarios but they are very relevant in the following cases:

- The process exits without a useful stack trace

- The process runs out of memory

- An application doesn’t behave as expected

The traditional problems with core dumps are: 

- Overhead of managing the dumps

- Dump Analysis required specific tooling that wasn't readily available on the developers machine.

- Managing Access to the dumps as they can contain sensitive information.

This chart aims to tackle the problems surrounding core dumps by leveraging common platforms (K8s, ROKS and Object Storage) in a cloud environment to pick up the heavy lifting.

## Chart Details

This chart utilizes a set of [simple bash scripts](https://github.com/No9/coredump-node-detector/tree/containerd-support/src) based on work by [Guangwen Feng](https://github.com/fenggw-fnst/coredump-node-detector). The scripts manage an install of a `/proc/sys/kernel/core_pattern` and a script to handle a core dump from a privileged container into the host server. A shared Cloud Object Store filesystem is also created between the privileged container and the host for deployment purposes and to allow the host to persist the coredumps "off box".

Information that is stored with IBM Cloud Object Storage is encrypted in transit and at rest, dispersed across multiple geographic locations, and accessed over HTTP by using a REST API.

When you install the IBM Cloud Core Dump Handler Helm chart, the following Kubernetes resources are deployed into your Kubernetes cluster:

- **Namespace**: A specific namespace is created to install the components into - defaults to ibm-observe

- **Handler Daemonset**: The daemonset deploys [pod](https://github.com/No9/coredump-node-detector/tree/containerd-support/src) on every worker node in your cluster. The daemonset contains scripts to define the core pattern on the host along with scripts to place the core dump into object storage as well as gather pod information if available.

- **Privileged Policy**: The daemonset configures the host node so priviledges are required.

- **Service Account**: Standard Service account to run the daemonset

- **Volume Claims**: For Timezone configuration, copying the coredump script to the host and integrating cloud object storage

- **Cluster Role**: Created with an **event** resource and **create** verb and associated with the service account. 

## Component Diagram
![Component Diagram](assets/topology.png)
## Prerequisites

[An IBM Cloud account](https://cloud.ibm.com/login)

[Virtual Routing and Forwarding Enabled](https://cloud.ibm.com/docs/account?topic=account-vrf-service-endpoint)

[An IBM Kubernetes Service Instance](https://cloud.ibm.com/kubernetes/catalog/create) **OR** [An RedHat OpenShift Kubernetes Service Instance](https://cloud.ibm.com/kubernetes/catalog/create?platformType=openshift)

[ibmcloud cli](https://cloud.ibm.com/docs/cli?topic=cloud-cli-install-ibmcloud-cli) with the following plugins
```
$ ibmcloud plugin install cloud-object-storage
$ ibmcloud plugin install kubernetes-service
```

[jq cli](https://stedolan.github.io/jq/download/) - Used in the `install-cos.sh` script 

if you are deploying into Openshift [oc](https://mirror.openshift.com/pub/openshift-v4/clients/oc/) is also required as well as the IBM Cloud CLI.



### Permissions
To install the Helm chart in your cluster, you must have the **Administrator** platform role.

## Security implications
This chart deploys privileged kubernetes daemon-set. The implications are the automatic creation of privileged container per kubernetes node capable of reading core files querying the crictl for pod info. The daemon-set also uses hostpath feature interacting with the underlying Linux OS.

## Resources Required
The IBM Cloud Core Dump Handler requires the following resources on each worker node to run successfully:
- CPU: 0.2 vCPU
- Memory: 128MB

## Installing the Chart

### Before you begin

If you are just starting out then make sure you have [VRF enabled ](https://cloud.ibm.com/docs/account?topic=account-vrf-service-endpoint) and a [cluster provisioned](https://cloud.ibm.com/kubernetes/catalog/create).

If you're taking the default install then take the following steps:

1. Set up the connection to your target cluster
`ibmlcoud ks cluster config -c YOURCLUSTER` or `$ oc login --token=XXX --server=https://XXX`

2. Run the install cos scipt
`./install-cos.sh

If you require specific configuration then [IBM Cloud Object Storage](https://hub.helm.sh/charts/ibm-charts/ibm-object-storage-plugin) can be configured outside of this script and the properties set in this charts `values.yaml` 

### Installing the chart

```
helm install coredump-handler . --namespace ibm-observe --set pvc.bucketName=A_UNIQUE_NAME
```

### Verifying the chart

1. Create a container 
```
$ kubectl run -i -t busybox --image=busybox --restart=Never
```
2. Login to the container
```
$ kubectl exec -it busybox -- /bin/sh
```
3. Generate a core dump by sending SIGSEGV to the terminal process.
```
# kill -11 $$
```
4. View the core dump tar file in the configured Cloud Object Store service instance.

## Removing the Chart

```
helm delete coredump-handler -n ibm-observe
```