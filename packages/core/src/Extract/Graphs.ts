export type GraphNode = {
  id: string;
};

export type GraphEdge<NodeType extends GraphNode> = {
  from: NodeType;
  to: NodeType;
};

export type NodeVisitor<NodeType> = (block: NodeType, parent?: NodeType) => void;

export abstract class Graph<
  NodeType extends GraphNode,
  EdgeType extends GraphEdge<NodeType> = GraphEdge<NodeType>
> {
  root: NodeType;
  protected nodeIndex: Map<string, NodeType>;

  constructor(root: NodeType) {
    this.root = root;

    this.nodeIndex = new Map<string, NodeType>();
    this.depthFirstSearch((node) => {
      this.nodeIndex.set(node.id, node);
    });
  }

  abstract getOutwardEdges(node: NodeType): EdgeType[];
  abstract getInwardEdges(node: NodeType): EdgeType[];

  depthFirstSearch(visitor: NodeVisitor<NodeType>): void {
    this.depthFirstSearchInner(this.root, visitor, new Set<string>());
  }

  generateGraphviz(): string {
    let code = 'digraph {\n';
    this.depthFirstSearch((node) => {
      this.getOutwardEdges(node).forEach((edge) => {
        code += `  ${this.getNodeName(node)} -> ${this.getNodeName(
          edge.to,
        )} [label="${this.getEdgeLabel(edge)}"]\n`;
      });
    });
    code += '}\n';
    return code;
  }

  getNodeName(node: NodeType): string {
    return node.id;
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  getEdgeLabel(edge: EdgeType): string {
    return '';
  }

  getNode(id: string): NodeType | undefined {
    return this.nodeIndex.get(id);
  }

  private depthFirstSearchInner(
    node: NodeType,
    visitor: NodeVisitor<NodeType>,
    visited: Set<string>,
    parent?: NodeType,
  ) {
    if (visited.has(node.id)) {
      return;
    }

    visitor(node, parent);
    visited.add(node.id);
    this.getOutwardEdges(node).forEach((exitEdge) => {
      this.depthFirstSearchInner(exitEdge.to, visitor, visited, node);
    });
  }
}
