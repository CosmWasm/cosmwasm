{% for doc in docs %}
    #[doc = "{{ doc }}"]
{% endfor %}

#[cosmwasm_schema::cw_serde]
pub struct {{ name }}

{% match ty %}
    {% when TypeTemplate::Unit %}
        ;
    {% when TypeTemplate::Tuple with (types) %}
        (
            {% for ty in types %}
                {{ ty }},
            {% endfor %}
        );
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
