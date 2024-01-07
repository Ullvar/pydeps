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

unknown_imports = []
with open('unknown_imports.json', 'r') as file:
    unknown_imports = json.load(file)

app.layout = html.Div(
    children=[
        html.H1(children='Dependencies', style={'textAlign': 'center'}),
        html.A(children=f'There was {len(unknown_imports)} files with unknown imports. Scroll down to see them.', style={'textAlign': 'center', 'display': 'block'}),
        dcc.Input(id='node-search', type='text', placeholder='Search for a node...', style={'marginTop': '30px', 'marginBottom': '20px', 'borderRadius': '5px', 'border': '1px solid #AEB6BF'}),
        html.Div(
            children=[
                html.Div(
                    style={'border': '1px solid black', 'padding': '10px', 'marginBottom': '5px'},
                    children=[
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
                            style={'width': '100%', 'height': '80vh'},
                            stylesheet=stylesheet
                        ),
                    ],
                ),
                
                # Unknown imports list
            ],
        ),
        html.H2("Unknown Imports"),
        html.Div([
            html.Div(
                style={'border': '1px solid black', 'padding': '10px', 'marginBottom': '5px', 'width': '50%'},
                children=[
                html.Ul([html.Li(imp) for imp in item['imports']]),
                html.H3(item['file_path']),
        ]) for item in unknown_imports
    ])
    ],
)


if __name__ == '__main__':
    app.run(debug=True)
