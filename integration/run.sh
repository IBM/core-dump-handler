#! /bin/bash

cd ../
export $(grep -v '^#' .env | xargs)

cd ./charts

helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3BucketName=${S3_BUCKET_NAME} --set daemonset.s3Region=${S3_REGION} \
--set daemonset.s3AccessKey=${S3_ACCESS_KEY} --set daemonset.s3Secret=${S3_SECRET} --set daemonset.interval=5000

## Poll until pod is up
while [[ $(kubectl get pods -n observe -l name=core-dump-ds -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; 
do 
    echo "waiting for pod" && sleep 1; 
done

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

if [ $file_count == "7" ]
then 
    echo -e "${GREEN}Success${NC}"
    cd ..
    rm -fr output
else
    echo -e "${RED}Failed${NC}"
    echo "expected 6 files but found ${file_count}"
    echo "Examine the output folder"
fi

helm delete -n observe core-dump-handler
