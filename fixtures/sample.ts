/**
 * This is a module-level documentation
 */

// Import statements
import { type } from "os";
import { EventEmitter } from "events";

// Type aliases
type PublicType = string;
type _PrivateType = number;

// Constants
const PUBLIC_CONSTANT: string = "constant";
const _PRIVATE_CONSTANT: number = 42;

// Enums
enum PublicEnum {
  First = "first",
  Second = "second"
}

enum _PrivateEnum {
  First = 1,
  Second = 2
}

// Interfaces
interface BaseInterface {
  abstractMethod(): void;
}

interface PublicInterface extends BaseInterface {
  publicField: string;
  publicMethod(param: string): string;
}

interface _PrivateInterface {
  _privateField: number;
  _privateMethod(): number;
}

// Classes
abstract class BaseClass {
  abstract abstractMethod(): void;
}

class PublicClass extends BaseClass implements PublicInterface {
  public publicField: string;
  private _privateField: number;

  constructor(publicField: string, privateField: number) {
    super();
    this.publicField = publicField;
    this._privateField = privateField;
  }

  public publicMethod(param: string): string {
    return `Hello ${param}`;
  }

  private _privateMethod(): number {
    return this._privateField;
  }

  abstractMethod(): void {
    console.log("Implemented abstract method");
  }
}

// Generic class
class GenericClass<T> {
  constructor(private value: T) { }

  getValue(): T {
    return this.value;
  }
}

// Functions
function publicFunction(param: string): string {
  return `Hello ${param}`;
}

function _privateFunction(): number {
  return 42;
}

// Arrow functions
const publicArrowFunction = (param: string): string => `Hello ${param}`;
const _privateArrowFunction = (): number => 42;

// Async functions
async function asyncFunction(): Promise<string> {
  return "Hello";
}

// Decorators
function decorator(target: any, propertyKey: string) {
  console.log(`Decorated ${propertyKey}`);
}

@decorator
class DecoratedClass {
  @decorator
  decoratedMethod() { }
}

// Type assertions
const value = "hello" as string;
const anotherValue = <string>"hello";

// Optional chaining
const obj = { prop: { nested: "value" } };
const nestedValue = obj?.prop?.nested;

// Nullish coalescing
const defaultValue = null ?? "default";

// Template literals
const name = "World";
const greeting = `Hello ${name}`;

// Destructuring
const { prop1, prop2 } = { prop1: "value1", prop2: "value2" };
const [first, second] = [1, 2];

// Spread operator
const arr1 = [1, 2, 3];
const arr2 = [...arr1, 4, 5];

// Rest parameters
function restFunction(...args: number[]): number {
  return args.reduce((a, b) => a + b, 0);
}

// Export statements
export { PublicClass, PublicInterface, PublicEnum, publicFunction };
export default PublicClass;
