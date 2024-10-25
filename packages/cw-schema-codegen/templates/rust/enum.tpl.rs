{% for doc in docs %}
    #[doc = "{{ doc }}"]
{% endfor %}

pub enum {{ name }} {
    {% for variant in variants %}
        {% for doc in variant.docs %}
            #[doc = "{{ doc }}"]
        {% endfor %}

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
                        {% for doc in field.docs %}
                            #[doc = "{{ doc }}"]
                        {% endfor %}

                        {{ field.name }}: {{ field.ty }},
                    {% endfor %}
                }
        {% endmatch %}
        ,
    {% endfor %}
}