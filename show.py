from dash import Dash, html, dcc, Input, Output, State
import dash_cytoscape as cyto
import json

app = Dash(__name__)

@app.callback(
    Output('cytoscape-graph', 'stylesheet', allow_duplicate=True),
    Input('node-search', 'value'),
    State('cytoscape-graph', 'stylesheet'),
    prevent_initial_call=True
)
def update_stylesheet(search_value, original_stylesheet):
    if not search_value:
        new_stylesheet = original_stylesheet[:]

        new_stylesheet.append({
            'selector': 'node',
            'style': {
                'background-color': '#00ffa5',
                'label': 'data(id)'
            }
        })
        new_stylesheet.append({
            'selector': '[id $= ".py"]',
            'style': {
                'background-color': '#005aff'
            }
        })

        return new_stylesheet

    new_stylesheet = original_stylesheet[:]

    new_stylesheet.append({
        'selector': 'node',
        'style': {
            'background-color': '#00ffa5',
            'label': 'data(id)'
        }
    })
    new_stylesheet.append({
        'selector': '[id $= ".py"]',
        'style': {
            'background-color': '#005aff'
        }
    })

    for element in elements:
        node_id = element.get('data', {}).get('id', '')
        if search_value.lower() in node_id.lower():
            highlight_style = {
                'selector': f'[id = "{node_id}"]',
                'style': {
                    'background-color': '#ffa500',
                    'border-color': 'black',
                    'border-width': 2
                }
            }
            new_stylesheet.append(highlight_style)

    return new_stylesheet

@app.callback(
    Output('cytoscape-graph', 'stylesheet', allow_duplicate=True),
    Input('cytoscape-graph', 'tapNodeData'),
    State('cytoscape-graph', 'stylesheet'),
    prevent_initial_call=True
)
def highlight_node(node_data, original_stylesheet):
    if not node_data:
        return original_stylesheet

    node_id = node_data['id']
    new_stylesheet = original_stylesheet[:]

    edge_style = {
        'selector': f'edge[source = "{node_id}"], edge[target = "{node_id}"]',
        'style': {
            'line-color': '#ff005a',
            'width': 3
        }
    }

    new_stylesheet.append(edge_style)
    return new_stylesheet


stylesheet=[
    {
        'selector': 'node',
        'style': {
            'background-color': '#00ffa5',
            'label': 'data(id)'
        }
    },
    {
        'selector': '[id $= ".py"]',
        'style': {
            'background-color': '#005aff'
        }
    },
    {
        'selector': 'edge',
        'style': {
            'line-color': '#AEB6BF'
        }
    }
]

elements = []
with open("graph_data.json", 'r') as file:
    elements = json.load(file)

app.layout = html.Div([
    html.H1(children='Dependencies', style={'textAlign':'center'}),
    dcc.Input(id='node-search', type='text', placeholder='Search for a node...'),
    cyto.Cytoscape(
        id='cytoscape-graph',
        elements=elements,
        layout={
            'name': 'cose',
            'idealEdgeLength': 300,
            'nodeOverlap': 20,
            'refresh': 20,
            'fit': True,
            'padding': 50,
            'randomize': False,
            'componentSpacing': 100,
            'nodeRepulsion': 400000,
            'edgeElasticity': 100,
            'nestingFactor': 5,
        },
        style={'width': '100vw', 'height': '100vh'},
        stylesheet=stylesheet
    )
])


if __name__ == '__main__':
    app.run(debug=True)
