# This code is @generated by cw-schema-codegen. Do not modify this manually.

import typing
from pydantic import BaseModel, RootModel

class {{ name }}(RootModel):
    """{% for doc in docs %}
    {{ doc }}
    {% endfor %}"""

{% for variant in variants %}
{% match variant.ty %}
{% when TypeTemplate::Unit %}
    class {{ variant.name }}(RootModel):
        """{% for doc in variant.docs %}
        {{ doc }}
        {% endfor %}"""
        root: None
{% when TypeTemplate::Tuple with (types) %}
    class {{ variant.name }}(BaseModel):
        """{% for doc in variant.docs %}
        {{ doc }}
        {% endfor %}"""
        {{ variant.name }}: typing.Tuple[{{ types|join(", ") }}]
{% when TypeTemplate::Named with { fields } %}
    class __Inner:
        """{% for doc in variant.docs %}
        {{ doc }}
        {% endfor %}"""
        {% for field in fields %}
        {{ field.name }}: {{ field.ty }}
        """{% for doc in field.docs %}
        # {{ doc }}
        {% endfor %}"""
        {% endfor %}
        {{ variant.name }}: __Inner
{% endmatch %}
{% endfor %}