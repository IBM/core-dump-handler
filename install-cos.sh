#/bin/sh
# Default script for provisining a core dump handler environment.
# See: https://github.com/No9/ibm-core-dump-handler

# Assumes login of ibmcloud is already performed and helm v3 is installed on the local machine

# Change this clustername to install on the specific cluster or pass it with -c
# e.g. ./install-cos.sh -c kcdt-test-002

NAMESPACE="ibm-observe"

for i in "$@"
do
case $i in
    -c=*|--CLUSTER_NAME=*)
        CLUSTER_NAME="${i#*=}"; shift

    -n=*|--NAMESPACE=*)
        NAMESPACE="${i#*=}"; shift
esac
done

# Using awk as the --json option is broken as it only returns one cluster
CLUSTER_ID=$(ibmcloud ks clusters ls | awk -v pat=$CLUSTER_NAME '{ if ( $1~pat) { print( $2 ) } }')
CLUSTER_VERSION=$(ibmcloud ks clusters ls | awk -v pat=$CLUSTER_NAME '{ if ( $1~pat) { print( $7 ) } }')
IS_ROKS=false

if [[ $CLUSTER_VERSION == *"openshift"* ]]; then
  IS_ROKS=true
fi


CLUSTER_INFO=$(ibmcloud ks cluster get -c $CLUSTER_ID --json)

RES_GRP=$(echo $CLUSTER_INFO | jq -r '.resourceGroup')
RES_GRP_NAME=$(echo $CLUSTER_INFO | jq -r '.resourceGroupName')

SERVICE_INSTANCE_NAME=$CLUSTER_NAME-cos-kcdt
SERVICE_INSTANCE_KEY=$CLUSTER_NAME-key-kcdt
ibmcloud resource service-instance-create $SERVICE_INSTANCE_NAME cloud-object-storage Standard global -g $RES_GRP_NAME 

ibmcloud resource service-key-create $SERVICE_INSTANCE_KEY 'Manager' --instance-name $SERVICE_INSTANCE_NAME -p "{\"HMAC\":true}"

SERVICE_CRED=$(ibmcloud resource service-key $SERVICE_INSTANCE_KEY --output json)

#cos_hmac_keys section, note the access_key_id and the secret_access_key.
ACCESS_KEY_ID=$(echo $SERVICE_CRED | jq -r '.[0].credentials.cos_hmac_keys.access_key_id')
SECRET_ACCESS_KEY=$(echo $SERVICE_CRED | jq -r '.[0].credentials.cos_hmac_keys.secret_access_key')

SERVICE_INSTANCE_ID=$(ibmcloud resource service-instance $SERVICE_INSTANCE_NAME | grep GUID)

if [ "$IS_ROKS" = true ] ; then
    oc new-project $NAMESPACE
    oc create secret generic cos-write-access --type=ibm/ibmc-s3fs --from-literal=access-key=$ACCESS_KEY_ID --from-literal=secret-key=$SECRET_ACCESS_KEY
    oc adm policy add-scc-to-user privileged -z coredump-admin
else
    kubectl create namespace $NAMESPACE
    kubectl create secret generic cos-write-access --type=ibm/ibmc-s3fs --from-literal=access-key=$ACCESS_KEY_ID --from-literal=secret-key=$SECRET_ACCESS_KEY -n $NAMESPACE
fi

helm repo add ibm-charts https://icr.io/helm/ibm-charts
helm repo update
helm fetch --untar ibm-charts/ibm-object-storage-plugin
helm plugin install ./ibm-object-storage-plugin/helm-ibmc

HELM_PLUGINS=$(helm env | grep HELM_PLUGINS | sed -e 's/HELM_PLUGINS=//g' | sed -e 's/"//g') 
chmod 755 $HELM_PLUGINS/helm-ibmc/ibmc.sh

helm ibmc install ibm-object-storage-plugin ibm-charts/ibm-object-storage-plugin --verbos