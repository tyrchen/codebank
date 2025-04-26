"""This is a module docstring"""

# Import statements
import os
from typing import List, Dict, Optional, Union, Any
from abc import ABC, abstractmethod

# Constants
PUBLIC_CONSTANT: str = "constant"
_PRIVATE_CONSTANT: int = 42

# Type aliases
PublicType = str
_PrivateType = int


# Classes
class BaseClass(ABC):
    """Base class with documentation"""

    @abstractmethod
    def abstract_method(self) -> None:
        """Abstract method documentation"""
        pass


class PublicClass(BaseClass):
    """Public class with documentation"""

    def __init__(self, public_field: str, private_field: int) -> None:
        """Constructor documentation"""
        self.public_field = public_field
        self._private_field = private_field

    def public_method(self, param: str) -> str:
        """Public method documentation"""
        return f"Hello {param}"

    def _private_method(self) -> int:
        """Private method documentation"""
        return self._private_field


class GenericClass[T]:
    """Generic class with documentation"""

    def __init__(self, value: T) -> None:
        self.value = value

    def get_value(self) -> T:
        return self.value


# Functions
def public_function(param1: str, param2: int = 0) -> str:
    """Public function documentation"""
    return f"{param1} {param2}"


def _private_function() -> None:
    """Private function documentation"""
    pass


# Decorators
def decorator(func):
    """Decorator documentation"""

    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)

    return wrapper


@decorator
def decorated_function() -> None:
    """Decorated function documentation"""
    pass


# Async functions
async def async_function() -> None:
    """Async function documentation"""
    pass


# Context managers
class ContextManager:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        pass


# Match statements
def match_example(value: int) -> str:
    match value:
        case 1:
            return "one"
        case 2:
            return "two"
        case _:
            return "other"


# Type hints
def type_hints_example(
    a: List[int], b: Dict[str, Any], c: Optional[str] = None, d: Union[int, str] = 0
) -> None:
    pass


# Properties
class PropertyExample:
    @property
    def value(self) -> int:
        return self._value

    @value.setter
    def value(self, new_value: int) -> None:
        self._value = new_value


# Class methods and static methods
class MethodExample:
    @classmethod
    def class_method(cls) -> None:
        pass

    @staticmethod
    def static_method() -> None:
        pass


# F-strings
def f_string_example(name: str) -> str:
    return f"Hello {name}"


# List/dict comprehensions
def comprehensions_example():
    squares = [x**2 for x in range(10)]
    squares_dict = {x: x**2 for x in range(10)}


# exception handling (not sure why tree-sitter-python doesn't support this)
# def exception_example():
#     try:
#         n = 0
#         res = 100 / n

#     except ZeroDivisionError:
#         print("You can't divide by zero!")

#     except ValueError:
#         print("Enter a valid number!")

#     else:
#         print("Result is", res)

#     finally:
#         print("Execution complete.")
