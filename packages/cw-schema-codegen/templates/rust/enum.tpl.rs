pub enum {{ name }} {
    {% for variant in variants %}
        {{ variant.name }}
        {% match variant.ty %}
            {% when TypeTemplate::Unit %}
            {% when TypeTemplate::Tuple with (types) %}
                (
                    {% for ty in types %}
                        {{ ty }},
                    {% endfor %}
                )
            {% when TypeTemplate::Named with { fields } %}
                {
                    {% for field in fields %}
                        {{ field.name }}: {{ field.ty }},
                    {% endfor %}
                }
        {% endmatch %}
        ,
    {% endfor %}
}