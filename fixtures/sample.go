// Package example is a sample Go file for testing the parser.
package example

import (
	"fmt"
	"io"
	"math"
	"os"
	"strings"
	"sync"
	"time"
)

// Constants
const (
	Pi = 3.14159
	MaxInt = 1<<63 - 1
)

// Variables
var (
	globalVar = "global"
	globalInt = 42
)

// PublicConst is an exported constant
const PublicConst = "public"

// privateConst is a non-exported constant
const privateConst = "private"

// Basic types
type BasicTypes struct {
	Bool bool
	Int int
	Int8 int8
	Int16 int16
	Int32 int32
	Int64 int64
	Uint uint
	Uint8 uint8
	Uint16 uint16
	Uint32 uint32
	Uint64 uint64
	Float32 float32
	Float64 float64
	Complex64 complex64
	Complex128 complex128
	String string
	Byte byte
	Rune rune
}

// Custom types
type CustomInt int
type CustomString string

// Interface
type Reader interface {
	Read(p []byte) (n int, err error)
}

// Implementation of Reader interface
type FileReader struct {
	file *os.File
}

func (fr *FileReader) Read(p []byte) (n int, err error) {
	return fr.file.Read(p)
}

// Struct with methods
type Point struct {
	X, Y float64
}

func (p Point) Distance() float64 {
	return math.Sqrt(p.X*p.X + p.Y*p.Y)
}

func (p *Point) Scale(factor float64) {
	p.X *= factor
	p.Y *= factor
}

// Function with multiple return values
func divide(a, b float64) (float64, error) {
	if b == 0 {
		return 0, fmt.Errorf("division by zero")
	}
	return a / b, nil
}

// Function with named return values
func split(sum int) (x, y int) {
	x = sum * 4 / 9
	y = sum - x
	return
}

// Function with variadic parameters
func sum(nums ...int) int {
	total := 0
	for _, num := range nums {
		total += num
	}
	return total
}

// Method with pointer receiver
func (p *Point) Move(dx, dy float64) {
	p.X += dx
	p.Y += dy
}

// Goroutine example
func say(s string) {
	for i := 0; i < 5; i++ {
		time.Sleep(100 * time.Millisecond)
		fmt.Println(s)
	}
}

// Channel example
func channelExample() {
	ch := make(chan int)
	go func() {
		ch <- 42
	}()
	value := <-ch
	fmt.Println(value)
}

// Select statement
func selectExample() {
	ch1 := make(chan string)
	ch2 := make(chan string)

	go func() { ch1 <- "one" }()
	go func() { ch2 <- "two" }()

	select {
	case msg1 := <-ch1:
		fmt.Println(msg1)
	case msg2 := <-ch2:
		fmt.Println(msg2)
	}
}

// Defer statement
func deferExample() {
	defer fmt.Println("world")
	fmt.Println("hello")
}

// Panic and recover
func panicExample() {
	defer func() {
		if r := recover(); r != nil {
			fmt.Println("Recovered:", r)
		}
	}()
	panic("a problem")
}

// Type switch
func typeSwitch(x interface{}) {
	switch x.(type) {
	case int:
		fmt.Println("int")
	case string:
		fmt.Println("string")
	default:
		fmt.Println("unknown")
	}
}

// Map example
func mapExample() {
	m := make(map[string]int)
	m["key"] = 42
	value, exists := m["key"]
	fmt.Println(value, exists)
}

// Slice example
func sliceExample() {
	s := make([]int, 5)
	s = append(s, 1, 2, 3)
	sub := s[1:3]
	fmt.Println(sub)
}

// Array example
func arrayExample() {
	var a [5]int
	a[0] = 1
	fmt.Println(a)
}

// Struct embedding
type Animal struct {
	Name string
}

type Dog struct {
	Animal
	Breed string
}

// Main function
func main() {
	// Basic control flow
	if x := 42; x > 0 {
		fmt.Println("x is positive")
	}

	for i := 0; i < 5; i++ {
		fmt.Println(i)
	}

	// Range loop
	nums := []int{1, 2, 3}
	for i, num := range nums {
		fmt.Println(i, num)
	}

	// Switch statement
	switch os := runtime.GOOS; os {
	case "darwin":
		fmt.Println("OS X")
	case "linux":
		fmt.Println("Linux")
	default:
		fmt.Println(os)
	}

	// Go routine and wait group
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		fmt.Println("goroutine")
	}()
	wg.Wait()
}

// Generic function
func Map[T, U any](slice []T, f func(T) U) []U {
    result := make([]U, len(slice))
    for i, v := range slice {
        result[i] = f(v)
    }
    return result
}

// Generic struct
type Container[T any] struct {
    Value T
}

// Generic method
func (c *Container[T]) Set(value T) {
    c.Value = value
}

func (c *Container[T]) Get() T {
    return c.Value
}

// Generic interface
type Stringer[T any] interface {
    String() string
}

// Generic type constraint
type Number interface {
    ~int | ~float64
}

// Generic function with type constraint
func Sum[T Number](numbers []T) T {
    var sum T
    for _, n := range numbers {
        sum += n
    }
    return sum
}

// Generic type with multiple type parameters
type Pair[T, U any] struct {
    First  T
    Second U
}

// Generic method with type constraint
func (p *Pair[T, U]) Swap() Pair[U, T] {
    return Pair[U, T]{
        First:  p.Second,
        Second: p.First,
    }
}

// Person represents a person with a name and age
type Person struct {
	// Name is the person's name
	Name string
	// Age is the person's age
	Age int
	// unexported field
	address string
}

// NewPerson creates a new Person instance
func NewPerson(name string, age int) *Person {
	return &Person{
		Name:    name,
		Age:     age,
		address: "unknown",
	}
}

// SetAddress sets the person's address
func (p *Person) SetAddress(address string) {
	p.address = address
}

// GetAddress returns the person's address
func (p *Person) GetAddress() string {
	return p.address
}

// String implements the Stringer interface
func (p Person) String() string {
	return fmt.Sprintf("%s (%d)", p.Name, p.Age)
}

// Greeter defines an interface for objects that can greet
type Greeter interface {
	// Greet returns a greeting message
	Greet() string
}

// GreeterImpl implements the Greeter interface
type GreeterImpl struct {
	greeting string
}

// Greet returns a greeting message
func (g GreeterImpl) Greet() string {
	return g.greeting
}

// UpperCase converts a string to uppercase
func UpperCase(s string) string {
	return strings.ToUpper(s)
}
