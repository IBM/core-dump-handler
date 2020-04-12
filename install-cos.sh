#/bin/sh
# Default script for provisining a core dump handler environment.
# See: https://github.com/No9/ibm-core-dump-handler

# Assumes login of ibmcloud is already performed and helm v3 is installed on the local machine

# Change this clustername to install on the specific cluster or pass it with -c
# e.g. ./install-cos.sh -c kcdt-test-002

set -e

function is_valid_value {
    if [[ ${1} == -* ]] || [[ ${1} == --* ]] || [[ -z ${1} ]]; then
        return 1
    else
        return 0
    fi
}

function help {
    echo "Usage: $(basename ${0}) [-n | --namespace | --project]"
    echo ""
    echo " -n  : if provided, will be the namespace used to deploy the agent. Defaults to ibm-observe"
    echo " -h  : print this usage and exit"
    echo
    exit 1
}

NAMESPACE="ibm-observe"
OPENSHIFT=0

while [[ ${#} > 0 ]]
do
key="${1}"

case ${key} in
    -n|--namespace|--project)
    if is_valid_value "${2}"; then
        NAMESPACE="${2}"
    else
        echo "ERROR: no valid value provided for namespace option, use -h | --help for $(basename ${0}) Usage"
        exit 1
    fi
    shift
    ;;
    -h|--help)
        help
        exit 1
        ;;
    *)
        echo "ERROR: Invalid option: ${1}, use -h | --help for $(basename ${0}) Usage"
        exit 1
        ;;
esac
shift
done

echo "* Retreiving the IKS Cluster ID and Cluster Name"
CLUSTER_ID=$(kubectl get cm -n kube-system cluster-info -o yaml | grep ' "cluster_id": ' | cut -d'"' -f4)
# Using awk as the --json option is broken as it only returns one cluster
CLUSTER_VERSION=$(ibmcloud ks clusters ls | awk -v pat=$CLUSTER_ID '{ if ( $2~pat) { print( $7 ) } }')
CLUSTER_NAME=$(ibmcloud ks clusters ls | awk -v pat=$CLUSTER_ID '{ if ( $2~pat) { print( $1 ) } }')

if [[ $CLUSTER_VERSION == *"openshift"* ]]; then
  OPENSHIFT=1
fi

CLUSTER_INFO=$(ibmcloud ks cluster get -c $CLUSTER_ID --json)

RES_GRP=$(echo $CLUSTER_INFO | jq -r '.resourceGroup')
RES_GRP_NAME=$(echo $CLUSTER_INFO | jq -r '.resourceGroupName')

SERVICE_INSTANCE_NAME=$CLUSTER_NAME-cos
SERVICE_INSTANCE_KEY=$CLUSTER_NAME-key
ibmcloud resource service-instance-create $SERVICE_INSTANCE_NAME cloud-object-storage Standard global -g $RES_GRP_NAME 

ibmcloud resource service-key-create $SERVICE_INSTANCE_KEY 'Manager' --instance-name $SERVICE_INSTANCE_NAME -p "{\"HMAC\":true}"

SERVICE_CRED=$(ibmcloud resource service-key $SERVICE_INSTANCE_KEY --output json)

#cos_hmac_keys section, note the access_key_id and the secret_access_key.
ACCESS_KEY_ID=$(echo $SERVICE_CRED | jq -r '.[0].credentials.cos_hmac_keys.access_key_id')
SECRET_ACCESS_KEY=$(echo $SERVICE_CRED | jq -r '.[0].credentials.cos_hmac_keys.secret_access_key')

out=""
fail=0
if [ $OPENSHIFT -eq 0 ]; then
    echo "* Creating namespace: $NAMESPACE"
    out=$(kubectl create namespace $NAMESPACE 2>&1) || { fail=1 && echo "kubectl create namespace failed!"; }
    kubectl create secret generic cos-write-access --type=ibm/ibmc-s3fs --from-literal=access-key=$ACCESS_KEY_ID --from-literal=secret-key=$SECRET_ACCESS_KEY -n $NAMESPACE
else
    echo "* Creating project: $NAMESPACE"
    out=$(oc adm new-project $NAMESPACE --node-selector='' 2>&1) || { fail=1 && echo "oc adm new-project failed!"; }
    # Set the project to the namespace
    switch=$(oc project $NAMESPACE 2>&1)
    oc create secret generic cos-write-access --type=ibm/ibmc-s3fs --from-literal=access-key=$ACCESS_KEY_ID --from-literal=secret-key=$SECRET_ACCESS_KEY
    oc adm policy add-scc-to-user privileged -z coredump-admin
fi

if [ $fail -eq 1 ]; then
    if [[ "$out" =~ "AlreadyExists" || "$out" =~ "already exists" ]]; then
        echo "$out. Continuing..."
    else
        echo "$out"
        exit 1
    fi
fi

out=""
fail=0
helm repo add ibm-charts https://icr.io/helm/ibm-charts
helm repo update
helm fetch --untar ibm-charts/ibm-object-storage-plugin
out=$(helm plugin install ./ibm-object-storage-plugin/helm-ibmc 2>&1) ||  { fail=1 && echo "helm plugin install failed!"; }
if [ $fail -eq 1 ]; then
    if [[ "$out" =~ "AlreadyExists" || "$out" =~ "already exists" ]]; then
        echo "$out. Continuing..."
    else
        echo "$out"
        exit 1
    fi
fi

HELM_PLUGINS=$(helm env | grep HELM_PLUGINS | sed -e 's/HELM_PLUGINS=//g' | sed -e 's/"//g') 
echo "Setting $HELM_PLUGINS/helm-ibmc/ibmc.sh to 755 Permissions"
chmod 755 $HELM_PLUGINS/helm-ibmc/ibmc.sh

helm ibmc install ibm-object-storage-plugin ibm-charts/ibm-object-storage-plugin --verbos

echo "helm install coredump-handler . --namespace $NAMESPACE --set pvc.bucketName=<unique_name>"
