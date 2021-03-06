// Copyright 2020 Walmart Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package main

import (
	"errors"
	"fmt"
	"github.com/golang/protobuf/proto"
	"github.com/hyperledger/sawtooth-sdk-go/messaging"
	zmq "github.com/pebbe/zmq4"
	"os"
	"protobuf/client_event_pb2"
	"protobuf/events_pb2"
	"protobuf/validator_pb2"
)

const (
	defaultValidatorURL = "tcp://validator:4004"
	// Calculated from the 1st 6 characters of SHA-512("cookiejar"):
	tpAddressPrefix = "ce2292"
)

// Global variable for remembering validator URL
var validatorToConnet = defaultValidatorURL

// Get URL from environment if present
func init() {
	validatorURL := os.Getenv("VALIDATOR_URL")
	if validatorURL != "" {
		validatorToConnet = validatorURL
	}
}

func listenToEvents(filters []*events_pb2.EventFilter) error {
	// Listen to cookiejar state-delta events.
	// Create a connection with validator for that
	zmqType := zmq.DEALER
	zmqContext, err := zmq.NewContext()
	if err != nil {
		return err
	}

	zmqConnection, err := messaging.NewConnection(zmqContext, zmqType, validatorToConnet, false)
	// Remember to close the connection when either not needed or error occurs
	if err != nil {
		return err
	}
	defer zmqConnection.Close()

	// Subscribe to events
	blockCommitSubscription := events_pb2.EventSubscription{
		EventType: "sawtooth/block-commit",
	}
	stateDeltaSubscription := events_pb2.EventSubscription{
		EventType: "sawtooth/state-delta",
		Filters:   filters,
	}
	request := client_event_pb2.ClientEventsSubscribeRequest{
		Subscriptions: []*events_pb2.EventSubscription{
			&blockCommitSubscription,
			&stateDeltaSubscription,
		},
	}
	serializedRequest, err := proto.Marshal(&request)
	if err != nil {
		return err
	}

	// Send the subscription request
	corrID, err := zmqConnection.SendNewMsg(
		validator_pb2.Message_CLIENT_EVENTS_SUBSCRIBE_REQUEST,
		serializedRequest,
	)
	if err != nil {
		return err
	}
	// Wait for subscription status
	_, response, err := zmqConnection.RecvMsgWithId(corrID)
	if err != nil {
		return err
	}
	eventSubscribeResponse := client_event_pb2.ClientEventsSubscribeResponse{}
	err = proto.Unmarshal(response.Content, &eventSubscribeResponse)
	if err != nil {
		return err
	}
	if eventSubscribeResponse.Status !=
		client_event_pb2.ClientEventsSubscribeResponse_OK {
		return errors.New("Client couldn't subscribe successfully")
	}

	// Listen for events in an infinite loop
	println("Listening to events.")
	for {
		_, message, err := zmqConnection.RecvMsg()
		if err != nil {
			return err
		}
		if message.MessageType != validator_pb2.Message_CLIENT_EVENTS {
			return errors.New("Unexpected message received, something which is not subscribed")
		}
		eventList := events_pb2.EventList{}
		err = proto.Unmarshal(message.Content, &eventList)
		if err != nil {
			return err
		}
		println("Received the following events: ----------")
		for _, event := range eventList.Events {
			fmt.Printf("Event: %v\n", *event)
		}
	}

	// Unsubscribe from events
	unSubscribeRequest := client_event_pb2.ClientEventsUnsubscribeRequest{}
	serializedRequest, err = proto.Marshal(&unSubscribeRequest)
	if err != nil {
		return err
	}
	corrID, err = zmqConnection.SendNewMsg(
		validator_pb2.Message_CLIENT_EVENTS_UNSUBSCRIBE_REQUEST,
		serializedRequest,
	)
	if err != nil {
		return err
	}
	// Wait for status
	_, unsubscribeResponse, err := zmqConnection.RecvMsgWithId(corrID)
	if err != nil {
		return err
	}
	eventUnsubscribeResponse := client_event_pb2.ClientEventsUnsubscribeResponse{}
	err = proto.Unmarshal(unsubscribeResponse.Content, &eventUnsubscribeResponse)
	if err != nil {
		return err
	}
	if eventUnsubscribeResponse.Status !=
		client_event_pb2.ClientEventsUnsubscribeResponse_OK {
		return errors.New("Client couldn't unsubscribe successfully")
	}
	return nil
}

func main() {
	// Entry point function for the client CLI.
	filters := []*events_pb2.EventFilter{&events_pb2.EventFilter{
		Key:         "address",
		MatchString: tpAddressPrefix + ".*",
		FilterType:  events_pb2.EventFilter_REGEX_ANY,
	}}
	// To listen to all events, there should not be any filters
	err := listenToEvents(filters)
	if err != nil {
		fmt.Printf("Error occurred %v\n", err)
	}
}
