package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"os"
)

func main() {
	var buffer bytes.Buffer

	_, err := io.Copy(&buffer, os.Stdin)
	if err != nil {
		panic(err)
	}

	data := buffer.Bytes()

	var out TestType
	if err := json.Unmarshal(data, &out); err != nil {
		panic(err)
	}

	fmt.Fprintln(os.Stderr, out)

	marshalled, err := json.Marshal(out)
	if err != nil {
		panic(err)
	}

	os.Stdout.Write(marshalled)
}
