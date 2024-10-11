pub enum {{ name }} {
    {% for variant in variants %}
        {{ variant.name }} {% if let Some(types) = variant.types %}({% for ty in types %} {{ ty }}, {% endfor %}) {% endif %},
    {% endfor %}
}