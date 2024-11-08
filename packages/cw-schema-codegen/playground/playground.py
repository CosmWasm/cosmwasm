from dataclasses import dataclass, field
from dataclasses_json import dataclass_json, config
from typing import Optional, Iterable
import sys
import json


# TODO tkulik: try to get rid of the `dataclasses_json` dependency


enum_field = lambda: field(default=None, metadata=config(exclude=lambda x: x is None))

@dataclass_json
@dataclass
class SomeEnum:
    class VariantIndicator:
        pass

    class Field3Type:
        a: str
        b: int

    class Field5Type:
        a: Iterable['SomeEnum']

    Field1: Optional[VariantIndicator] = enum_field()
    Field2: Optional[tuple[int, int]] = enum_field()
    Field3: Optional[Field3Type] = enum_field()
    Field4: Optional[Iterable['SomeEnum']] = enum_field()
    Field5: Optional[Field5Type] = enum_field()
    
    def deserialize(json):
        if not ":" in json:
            if json == '"Field1"':
                return SomeEnum(Field1=SomeEnum.VariantIndicator())
            else:
                raise Exception(f"Deserialization error, undefined variant: {json}")
        else:
            return SomeEnum.from_json(json)
        
    def serialize(self):
        if self.Field1 is not None:
            return '"Field1"'
        else:
            return SomeEnum.to_json(self)
        
@dataclass_json
@dataclass
class UnitStructure:
    def deserialize(json):
        if json == "null":
            return UnitStructure()
        else:
            Exception(f"Deserialization error, undefined value: {json}")
        
    def serialize(self):
        return 'null'

@dataclass_json
@dataclass
class TupleStructure:
    Tuple: tuple[int, str, int]

    def deserialize(json):
        return TupleStructure.from_json(f'{{ "Tuple": {json} }}')
        
    def serialize(self):
        return json.dumps(self.Tuple)

@dataclass_json
@dataclass
class NamedStructure:
    a: str
    b: int
    c: Iterable['SomeEnum']

    def deserialize(json):
        return NamedStructure.from_json(json)
        
    def serialize(self):
        return self.to_json()

###
### TESTS:
###

for (index, input) in enumerate(sys.stdin):
    input = input.rstrip()
    try:
        if index < 5:
            deserialized = SomeEnum.deserialize(input)
        elif index == 5:
            deserialized = UnitStructure.deserialize(input)
        elif index == 6:
            deserialized = TupleStructure.deserialize(input)
        else:
            deserialized = NamedStructure.deserialize(input)
    except:
        raise(Exception(f"This json can't be deserialized: {input}"))
    serialized = deserialized.serialize()
    print(serialized)


# def handle_msg(json):
#     a = SomeEnum.deserialize(json)
#     if a.Field1 is not None:
#         print("SomeEnum::Field1")
#     elif a.Field2 is not None:
#         print(a.Field2[0])
#         print(a.Field2[1])
#     elif a.Field3 is not None:
#         print(a.Field3)
#     elif a.Field4 is not None:
#         print(a.Field4)
#     elif a.Field5 is not None:
#         print(a.Field5)

# handle_msg('"Field1"')
# handle_msg('{"Field2": [10, 12]}')