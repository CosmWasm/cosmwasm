{% for doc in docs %}
    #[doc = "{{ doc }}"]
{% endfor %}

#[cosmwasm_schema::cw_serde]
pub enum {{ name }} {
    {% for variant in variants %}
        {% for doc in variant.docs %}
            #[doc = "{{ doc }}"]
        {% endfor %}

        {% match variant.serde_rename %}
            {% when Some with (rename) %}
                #[serde(rename = "{{ rename }}")]
            {% when None %}
        {% endmatch %}

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