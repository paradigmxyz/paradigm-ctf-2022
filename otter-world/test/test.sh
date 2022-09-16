cd .. && tar hczvf chall.tar.gz client

cd test
cp ../chall.tar.gz .

rm -rf client
tar xf chall.tar.gz
