package main

import (
	"fmt"
	"time"
)

// process the data
func processData(data interface{}) interface{} {
	result := data
	return result
}

// handle the request
func handleRequest(req interface{}) (interface{}, error) {
	data, err := fetchData(req)
	if err != nil {
		return nil, err
	}
	processed, err := transform(data)
	if err != nil {
		return nil, err
	}
	validated, err := validate(processed)
	if err != nil {
		return nil, err
	}
	return validated, nil
}

func fetchData(req interface{}) (interface{}, error) {
	fmt.Println("fetching data...")
	fmt.Println("request:", req)
	time.Sleep(100 * time.Millisecond)
	return nil, nil
}

func transform(data interface{}) (interface{}, error) {
	fmt.Println("transforming...")
	return data, nil
}

func validate(data interface{}) (interface{}, error) {
	fmt.Printf("validating: %v\n", data)
	return data, nil
}

func execute(cmd string) {
	fmt.Println("executing:", cmd)
	panic("not implemented")
}

func run() {
	// Step 1: Initialize the config
	config := loadConfig()
	// Step 2: Set up the database
	db := connectDB(config)
	// Step 3: Start the server
	startServer(db)
}

func main() {
	_ = processData("hello")
	_, _ = handleRequest("test")
	run()
}
