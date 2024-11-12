// This code is @generated by cw-schema-codegen. Do not modify this manually.

/**
{% for doc in docs %}
    * {{ doc }}
{% endfor %}
 */

type {{ name }} = 
{% for variant in variants %}
    | 

    /**
    {% for doc in variant.docs %}
        * {{ doc }}
    {% endfor %}
     */

    {% match variant.ty %}
        {% when TypeTemplate::Unit %}
            { "{{ variant.name }}": {} }
        {% when TypeTemplate::Tuple with (types) %}
            { "{{ variant.name }}": [{{ types|join(", ") }}] }
        {% when TypeTemplate::Named with { fields } %}
            { "{{ variant.name }}": {
                {% for field in fields %}
                    /**
                    {% for doc in field.docs %}
                        * {{ doc }}
                    {% endfor %}
                     */

                    {{ field.name }}: {{ field.ty }};
                {% endfor %}
            } }
    {% endmatch %}
{% endfor %}

{% if variants.len() == 0 %}
    never;
{% endif %}
;

export { {{ name }} };
