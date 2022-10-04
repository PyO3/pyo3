{% for section_text, section in sections.items() %}{%- if section %}{{section_text}}{% endif -%}
{% if section %}
{% for category in ['packaging', 'added', 'changed', 'removed', 'fixed' ] if category in section %}
### {{ definitions[category]['name'] }}

{% if definitions[category]['showcontent'] %}
{% for text, pull_requests in section[category].items() %}
- {{ text }} {{ pull_requests|join(', ') }}
{% endfor %}
{% else %}
- {{ section[category]['']|join(', ') }}
{% endif %}

{% endfor %}{% else %}No significant changes.{% endif %}{% endfor %}
