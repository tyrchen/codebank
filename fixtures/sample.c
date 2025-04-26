#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Preprocessor directives
#define MAX_SIZE 100
#define MIN(a, b) ((a) < (b) ? (a) : (b))

// Type definitions
typedef struct {
    int x;
    int y;
} Point;

typedef enum {
    RED,
    GREEN,
    BLUE
} Color;

// Function declarations
void print_hello(void);
int add_numbers(int a, int b);
void process_array(int arr[], int size);
void handle_pointers(void);
void use_control_flow(void);
void demonstrate_memory_allocation(void);

// Main function
int main(void) {
    print_hello();
    int sum = add_numbers(5, 3);
    printf("Sum: %d\n", sum);

    int numbers[] = {1, 2, 3, 4, 5};
    process_array(numbers, 5);

    handle_pointers();
    use_control_flow();
    demonstrate_memory_allocation();

    return 0;
}

// Function definitions
void print_hello(void) {
    printf("Hello, World!\n");
}

int add_numbers(int a, int b) {
    return a + b;
}

void process_array(int arr[], int size) {
    for (int i = 0; i < size; i++) {
        printf("Element %d: %d\n", i, arr[i]);
    }
}

void handle_pointers(void) {
    int x = 10;
    int *ptr = &x;
    int **pptr = &ptr;

    printf("Value of x: %d\n", x);
    printf("Value through pointer: %d\n", *ptr);
    printf("Value through pointer to pointer: %d\n", **pptr);
}

void use_control_flow(void) {
    // If-else
    int x = 10;
    if (x > 5) {
        printf("x is greater than 5\n");
    } else {
        printf("x is less than or equal to 5\n");
    }

    // Switch
    Color color = RED;
    switch (color) {
        case RED:
            printf("Color is red\n");
            break;
        case GREEN:
            printf("Color is green\n");
            break;
        case BLUE:
            printf("Color is blue\n");
            break;
        default:
            printf("Unknown color\n");
    }

    // Loops
    for (int i = 0; i < 5; i++) {
        printf("For loop iteration %d\n", i);
    }

    int j = 0;
    while (j < 5) {
        printf("While loop iteration %d\n", j);
        j++;
    }

    do {
        printf("Do-while loop iteration %d\n", j);
        j++;
    } while (j < 10);
}

void demonstrate_memory_allocation(void) {
    // Static allocation
    int static_array[10];

    // Dynamic allocation
    int *dynamic_array = (int *)malloc(10 * sizeof(int));
    if (dynamic_array == NULL) {
        printf("Memory allocation failed\n");
        return;
    }

    // Use allocated memory
    for (int i = 0; i < 10; i++) {
        dynamic_array[i] = i * 2;
    }

    // Reallocate memory
    dynamic_array = (int *)realloc(dynamic_array, 20 * sizeof(int));

    // Free memory
    free(dynamic_array);
}
