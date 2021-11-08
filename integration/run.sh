#! /bin/bash

cd ../
export $(grep -v '^#' .env | xargs)

cd ./charts/core-dump-handler

helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3BucketName=${S3_BUCKET_NAME} --set daemonset.s3Region=${S3_REGION} \
--set daemonset.s3AccessKey=${S3_ACCESS_KEY} --set daemonset.s3Secret=${S3_SECRET}
## Poll until pod is up
while [[ $(kubectl get pods -n observe -l name=core-dump-ds -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; 
do 
    echo "waiting for core dump pod to be setup" && sleep 1; 
done

echo "Core dump pod is ready - starting crash test"

kubectl run -it segfaulter --image=quay.io/icdh/segfaulter --restart=Never

kubectl delete pod segfaulter

mc alias set storage https://$S3_REGION $S3_ACCESS_KEY $S3_SECRET

while [[ $(mc find storage/${S3_BUCKET_NAME} --newer-than 1m) == "" ]]; 
do 
    echo "waiting for upload to complete" && sleep 1;
done

file_name=$(mc find storage/${S3_BUCKET_NAME} --newer-than 1m)
base_name=$(basename ${file_name})
rm -fr ./output
mkdir ./output 
cd ./output

echo "Copying $base_name"
mc cp $file_name .

unzip $base_name 

file_count=$(ls | wc -l)

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

node_hostname=$(jq -r '.node_hostname' *-dump-info.json)

if [[ $node_hostname ]];
then
    echo -e "${GREEN}Success: Node Name Exists${NC}"
    cd ..
    rm -fr output
else
    echo -e "${RED}Failed${NC}"
    echo "Node Does NOT Name Exists ${node_hostname}"
    echo "Examine the output folder"
fi

if [ $file_count == "7" ]
then
    echo -e "${GREEN}Sucess: Correct File Count${NC}"
    cd ..
    rm -fr output
else
    echo -e "${RED}Failed${NC}"
    echo "expected 6 files but found ${file_count}"
    echo "Examine the output folder"
fi

mc rm $file_name
helm delete -n observe core-dump-handler
