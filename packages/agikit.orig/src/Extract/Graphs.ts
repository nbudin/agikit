export type GraphEdge<NodeType> = {
  from: NodeType;
  to: NodeType;
};

export type NodeVisitor<NodeType> = (block: NodeType, parent?: NodeType) => void;

export abstract class Graph<NodeType> {
  root: NodeType;

  constructor(root: NodeType) {
    this.root = root;
  }

  abstract getOutwardEdges(node: NodeType): GraphEdge<NodeType>[];
  abstract getInwardEdges(node: NodeType): GraphEdge<NodeType>[];

  depthFirstSearch(visitor: NodeVisitor<NodeType>): void {
    this.depthFirstSearchInner(this.root, visitor, new Set<NodeType>());
  }

  private depthFirstSearchInner(
    node: NodeType,
    visitor: NodeVisitor<NodeType>,
    visited: Set<NodeType>,
    parent?: NodeType,
  ) {
    if (visited.has(node)) {
      return;
    }

    visitor(node, parent);
    visited.add(node);
    this.getOutwardEdges(node).forEach((exitEdge) => {
      this.depthFirstSearchInner(exitEdge.to, visitor, visited, node);
    });
  }
}
