Go Spinner
==========

usage
-----

```go
func UsageShowcase() {
	s := spin.New("Processing: ")
	s.Start()
	time.Sleep(10 * time.Second)
	s.Stop()
	fmt.Println("Done!")
}
```
