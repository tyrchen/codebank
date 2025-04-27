#include <string>
#include <vector>

// A sample class with fields in different access specifiers
class MyClass {
public:
    /**
     * Public integer data.
     * Can be accessed from anywhere.
     */
    int public_data;

    // Constructor
    MyClass(int data) : public_data(data), protected_flag(true), private_name("default") {}

    void public_method() {
        // Method body
    }

protected:
    bool protected_flag;

private:
    /**
     * Private string name.
     */
    std::string private_name;

    void private_method() {
        // Method body
    }
};

// A simple struct (implicitly public fields)
struct MyStruct {
    double x;
    double y;
};
