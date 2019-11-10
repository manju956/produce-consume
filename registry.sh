#!/bin/bash

# Copyright 2019 Walmart Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

value=`cat /keys/validator.pub`
echo $value

# Create the contract registry
sabre cr --create produce-consume --owner $value --url http://rest-api:8008 --key /keys/validator

# Upload the contract definition language
sabre upload --filename contract-definition.yaml --url http://rest-api:8008 --key /keys/validator

# Create namespace registry and set contract permissions
sabre ns --create ce2292 --owner $value --url http://rest-api:8008 --key /keys/validator

sabre perm ce2292 produce-consume --read --write --url http://rest-api:8008 --key /keys/validator

sabre ns --create cad11d --owner $value --url http://rest-api:8008 --key /keys/validator

sabre perm cad11d produce-consume --read --url http://rest-api:8008 --key /keys/validator
