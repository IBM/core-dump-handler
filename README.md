# IBM Core Dump Handler

This helm chart creates to automatically save core dumps from [IBM Kubernetes Service](https://cloud.ibm.com/docs/containers?topic=containers-getting-started) pods to [IBM Cloud Object Storage](https://cloud.ibm.com/docs/services/cloud-object-storage?topic=cloud-object-storage-about-ibm-cloud-object-storage#about-ibm-cloud-object-storage).


## Introduction

[Core Dumps](https://en.wikipedia.org/wiki/Core_dump) are a critical part of observability.

As systems become more distributed core dumps offer teams a non-invasive approach to understanding why programs are malfunctioning in any environment they are deployed to. 

This chart utilizes a set of [simple bash scripts](https://github.com/No9/coredump-node-detector/tree/containerd-support/src) based on work by [Guangwen Feng](https://github.com/fenggw-fnst/coredump-node-detector). The scripts manage an install of a core_pattern and script to handle a core dump from a privileged container into the host server. A shared Cloud Object Store filesystem is also created between the privileged container and the host for deployment purposes and to allow the host to persist the coredumps "off box".

Information that is stored with IBM Cloud Object Storage is encrypted in transit and at rest, dispersed across multiple geographic locations, and accessed over HTTP by using a REST API.

## Chart Details
When you install the IBM Cloud Core Dump Handler Helm chart, the following Kubernetes resources are deployed into your Kubernetes cluster:

- **IBM Cloud Kubernetes Core Dump Tool daemonset**: The daemonset deploys one `kcdt` pod on every worker node in your cluster. The daemonset contains scripts to define the core pattern on the host along with scripts to place the core dump into object storage as well as gather pod information if available.

DEPLOYMENT ARCH HERE.

RUNTIME ARCH HERE.

## Prerequisites

[An IBM Cloud account]()

[Virtual Routing and Forwarding Enabled](https://cloud.ibm.com/docs/account?topic=account-vrf-service-endpoint)

[An IBM Kubernetes Service Instance](https://cloud.ibm.com/kubernetes/catalog/create)

### Permissions
To install the Helm chart in your cluster, you must have the **Administrator** platform role.

## Resources Required
The IBM Cloud Core Dump Handler requires the following resources on each worker node to run successfully:
- CPU: 0.2 vCPU
- Memory: 128MB

## Installing the Chart

### Before you begin

If you are just starting out then make sure you have [VRF enabled ](https://cloud.ibm.com/docs/account?topic=account-vrf-service-endpoint) and a [test cluster provisioned](https://cloud.ibm.com/kubernetes/catalog/create).

You can then use [This script](https://gist.github.com/No9/3cf779bed538ee3e89e564a32512da09) to provide the default COS plugin install and configuration as well as creating the keys required to just run the chart. 

Provision and configure the Cloud Object Store Kubernetes plugin.

https://hub.helm.sh/charts/ibm-charts/ibm-object-storage-plugin 


Create a token
```
$ ibmcloud resource service-key-create $SERVICE_INSTANCE_KEY 'Manager' \ 
    --instance-name $SERVICE_INSTANCE_NAME -p "{\"HMAC\":true}"
```

Get the token information
```
$ ibmcloud resource service-key $SERVICE_INSTANCE_KEY 
```

Create the namespace
```
$ create namespace ibm-observe
```

Store the token as a secret in the namespace
```
$ kubectl create secret generic cos-write-access --type=ibm/ibmc-s3fs \
--from-literal=access-key= --from-literal=secret-key= -n ibm-observe
```

Update the pvc section of values created above
```
pvc:
    bucketName: "coredumps-002" #name of the bucket
    bucketSecretName: "cos-write-access" #unless you changed the secret name this should stay the same
```

### Installing the chart

Simply
```
helm install coredump-handler . --namespace ibm-observe 
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