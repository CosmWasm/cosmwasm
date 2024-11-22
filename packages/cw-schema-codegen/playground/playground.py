import sys
from typing import Literal, Union, Tuple
from pydantic import BaseModel, RootModel


class SomeEnum(RootModel):
    class Field1(RootModel[Literal['Field1']]):
        pass

    class Field2(BaseModel):
        Field2: Tuple[int, int]

    class Field3(BaseModel):
        class __InnerStruct(BaseModel):
            a: str
            b: int
        Field3: __InnerStruct

    class Field4(BaseModel):
        Field4: 'SomeEnum'

    class Field5(BaseModel):
        class __InnerStruct(BaseModel):
            a: 'SomeEnum'
        Field5: __InnerStruct

    root: Union[Field1, Field2, Field3, Field4, Field5]


class UnitStructure(RootModel):
    root: None


class TupleStructure(RootModel):
    root: Tuple[int, str, int]


class NamedStructure(BaseModel):
    a: str
    b: int
    c: SomeEnum



###
### TESTS:
###

for (index, input) in enumerate(sys.stdin):
    input = input.rstrip()
    try:
        if index < 5:
            deserialized = SomeEnum.model_validate_json(input)
        elif index == 5:
            deserialized = UnitStructure.model_validate_json(input)
        elif index == 6:
            deserialized = TupleStructure.model_validate_json(input)
        else:
            deserialized = NamedStructure.model_validate_json(input)
    except:
        raise(Exception(f"This json can't be deserialized: {input}"))
    serialized = deserialized.model_dump_json()
    print(serialized)


# def handle_msg(json):
#     a = SomeEnum.model_validate_json(json)
#     if isinstance(a.root, SomeEnum.Field1):
#         print("SomeEnum::Field1")
#     elif isinstance(a.root, SomeEnum.Field2):
#         print(a.root.Field2[0])
#         print(a.root.Field2[1])
#     elif isinstance(a.root, SomeEnum.Field3):
#         print(a.root.Field3)
#     elif isinstance(a.root, SomeEnum.Field4):
#         print(a.root.Field4)
#     elif isinstance(a.root, SomeEnum.Field5):
#         print(a.root.Field5)

# handle_msg('"Field1"')
# handle_msg('{"Field2": [10, 12]}')
# handle_msg('{"Field3": { "a": "10", "b": 12 } }')
# handle_msg('{"Field4": { "Field4": "Field1" } }')
# handle_msg('{"Field5": { "a": "Field1" } }')
# handle_msg('{"Field5": { "a": { "Field5": { "a": "Field1" } } } }')