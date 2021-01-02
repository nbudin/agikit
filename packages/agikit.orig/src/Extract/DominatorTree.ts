import { Graph } from './Graphs';

type SpanningTreeNodeInfo<NodeType> = {
  dfsNum: number;
  idom?: NodeType;
  sdom?: NodeType;
  parent?: NodeType;
  ancestor?: NodeType;
  best?: NodeType;
};

export type ImmediateDominatorEdge<CFGNodeType> = {
  type: 'immediateDominator';
  from: DominatorTreeNode<CFGNodeType>;
  to: DominatorTreeNode<CFGNodeType>;
};

export type DominatorTreeNode<CFGNodeType> = {
  cfgNode: CFGNodeType;
  immediateDominator?: ImmediateDominatorEdge<CFGNodeType>;
  dominates: ImmediateDominatorEdge<CFGNodeType>[];
};

export class DominatorTree<CFGNodeType> extends Graph<DominatorTreeNode<CFGNodeType>> {
  static fromCFG<CFGNodeType>(graph: Graph<CFGNodeType>): DominatorTree<CFGNodeType> {
    const spanningTree = new SemiNCASpanningTree(graph);
    const root = spanningTree.buildDominatorTree();
    return new DominatorTree(root);
  }

  private nodeIndex: Map<CFGNodeType, DominatorTreeNode<CFGNodeType>>;

  constructor(root: DominatorTreeNode<CFGNodeType>) {
    super(root);

    this.nodeIndex = new Map<CFGNodeType, DominatorTreeNode<CFGNodeType>>();
    this.depthFirstSearch((node) => {
      this.nodeIndex.set(node.cfgNode, node);
    });
  }

  dominates(a: CFGNodeType, b: CFGNodeType): boolean {
    const aNode = this.nodeIndex.get(a);
    let bNode = this.nodeIndex.get(b);

    if (!aNode || !bNode) {
      throw new Error('Node not in graph');
    }

    if (aNode === bNode) {
      return true;
    }

    while (bNode.immediateDominator) {
      if (bNode.immediateDominator.from === aNode) {
        return true;
      }
      bNode = bNode?.immediateDominator.from;
    }

    return false;
  }

  getInwardEdges(node: DominatorTreeNode<CFGNodeType>): ImmediateDominatorEdge<CFGNodeType>[] {
    if (node.immediateDominator) {
      return [node.immediateDominator];
    }

    return [];
  }

  getOutwardEdges(node: DominatorTreeNode<CFGNodeType>): ImmediateDominatorEdge<CFGNodeType>[] {
    return node.dominates;
  }
}

class SemiNCASpanningTree<NodeType> {
  graph: Graph<NodeType>;
  private nodesInDFSOrder: NodeType[];
  private nodeInfos: Map<NodeType, SpanningTreeNodeInfo<NodeType>>;

  constructor(graph: Graph<NodeType>) {
    this.graph = graph;
    this.nodeInfos = new Map<NodeType, SpanningTreeNodeInfo<NodeType>>();
    this.nodesInDFSOrder = [];

    graph.depthFirstSearch((block, parent) => {
      this.nodeInfos.set(block, { dfsNum: this.nodesInDFSOrder.length, parent, idom: parent });
      this.nodesInDFSOrder.push(block);
    });

    this.computeSemidominators();
    this.computeImmediateDominators();
  }

  buildDominatorTree(): DominatorTreeNode<NodeType> {
    const dtNodes = new Map<NodeType, DominatorTreeNode<NodeType>>(
      this.nodesInDFSOrder.map((node) => [node, { cfgNode: node, dominates: [] }]),
    );

    this.nodesInDFSOrder.forEach((node) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const dtNode = dtNodes.get(node)!;
      const immediateDominator = this.getImmediateDominator(node);
      if (immediateDominator) {
        const edge: ImmediateDominatorEdge<NodeType> = {
          type: 'immediateDominator',
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          from: dtNodes.get(immediateDominator)!,
          to: dtNode,
        };
        dtNode.immediateDominator = edge;
        edge.from.dominates.push(edge);
      }
    });

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    return dtNodes.get(this.graph.root)!;
  }

  private getNodeInfo(node: NodeType): SpanningTreeNodeInfo<NodeType> {
    const nodeInfo = this.nodeInfos.get(node);
    if (!nodeInfo) {
      throw new Error('Node is not in graph');
    }
    return nodeInfo;
  }

  private getDFSParent(node: NodeType): NodeType | undefined {
    return this.getNodeInfo(node).parent;
  }

  private getSpanningForestAncestor(node: NodeType): NodeType | undefined {
    return this.getNodeInfo(node).ancestor;
  }

  private getSemidominator(node: NodeType): NodeType | undefined {
    return this.getNodeInfo(node).sdom;
  }

  private getImmediateDominator(node: NodeType): NodeType | undefined {
    return this.getNodeInfo(node).idom;
  }

  private getBest(node: NodeType): NodeType | undefined {
    return this.getNodeInfo(node).best;
  }

  private getDFSNumber(node: NodeType): number {
    return this.getNodeInfo(node).dfsNum;
  }

  private ancestorWithLowestSemi(node: NodeType): NodeType | undefined {
    const ancestor = this.getSpanningForestAncestor(node);

    if (ancestor != null) {
      const candidateBest = this.ancestorWithLowestSemi(ancestor);
      this.getNodeInfo(node).ancestor = this.getSpanningForestAncestor(ancestor);

      const candidateBestSDom = candidateBest ? this.getSemidominator(candidateBest) : undefined;
      const currentBest = this.getBest(node);
      const currentBestSDom = currentBest ? this.getSemidominator(currentBest) : undefined;

      if (
        candidateBestSDom &&
        currentBestSDom &&
        this.getDFSNumber(candidateBestSDom) < this.getDFSNumber(currentBestSDom)
      ) {
        this.getNodeInfo(node).best = candidateBest;
      }
    }

    return this.getBest(node);
  }

  private link(ancestor: NodeType, node: NodeType) {
    const nodeInfo = this.getNodeInfo(node);
    nodeInfo.ancestor = ancestor;
    nodeInfo.best = node;
  }

  private computeSemidominators() {
    // iterate nodes in reverse DFS order, omitting the root
    const reversePreOrderNodes = this.nodesInDFSOrder.slice(1).reverse();
    reversePreOrderNodes.forEach((node) => {
      // we know there will be a parent because we're omitting the root
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const parent = this.getDFSParent(node)!;
      let semi: NodeType = parent;

      this.graph.getInwardEdges(node).forEach((inwardEdge) => {
        const predecessor = inwardEdge.from;

        let candidate: NodeType | undefined;
        if (this.getDFSNumber(predecessor) < this.getDFSNumber(node)) {
          candidate = predecessor;
        } else {
          const ancestorWithLowest = this.ancestorWithLowestSemi(predecessor);
          if (ancestorWithLowest != null) {
            candidate = this.getSemidominator(ancestorWithLowest);
          }
        }

        if (candidate && this.getDFSNumber(candidate) < this.getDFSNumber(semi)) {
          semi = candidate;
        }
      });

      this.getNodeInfo(node).sdom = semi;
      this.link(parent, node);
    });
  }

  private computeImmediateDominators() {
    // iterate nodes in reverse DFS order, omitting the root
    const preOrderNodes = this.nodesInDFSOrder.slice(1);
    preOrderNodes.forEach((node) => {
      // we know there's an idom for this node because we initialize it to the parent for all
      // nodes (except the root, for which there's no parent)
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      let immediateDominator = this.getImmediateDominator(node)!;
      const semidominator = this.getSemidominator(node);
      if (!semidominator) {
        throw new Error('computeSemidominators must be called before computeImmediateDominators');
      }
      const semidominatorDFSNum = this.getDFSNumber(semidominator);

      while (this.getDFSNumber(immediateDominator) > semidominatorDFSNum) {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        immediateDominator = this.getImmediateDominator(immediateDominator)!;
      }

      this.getNodeInfo(node).idom = immediateDominator;
    });
  }
}
