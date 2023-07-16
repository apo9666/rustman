import { type Dispatch, createContext, useReducer } from 'react'
import { type TabContent } from './TabContext'

export interface TreeNode {
  id: number
  label: string
  content?: TabContent
  expanded: boolean
  children?: TreeNode[]
}

type TreeAction =
  | { type: 'ADD_NODE', payload: { parentId: number, node: Omit<TreeNode, 'id'> } }
  | { type: 'EDIT_NODE', payload: { node: TreeNode } }
  | { type: 'REMOVE_NODE', payload: { node: TreeNode } }

interface TreeState {
  tree: TreeNode
  lastTreeId: number
}

const treeInitialState: TreeState = {
  tree: {
    id: 0,
    label: 'Root',
    expanded: false,
    children: [
      {
        id: 1,
        label: 'GET google/abc',
        content: {
          body: '',
          method: 'post',
          url: 'https://dummyjson.com/products/1'
        },
        expanded: false
      },
      {
        id: 2,
        label: 'Banana',
        expanded: false,
        children: [
          {
            id: 3,
            label: 'Get banana',
            expanded: false
          }
        ]
      }
    ]
  },
  lastTreeId: 0
}

const findNode = (node: TreeNode, id: number): TreeNode | undefined => {
  if (node.id === id) {
    return node
  }

  if (node.children === undefined) {
    return
  }

  for (const child of node.children) {
    const result = findNode(child, id)

    if (result !== undefined) {
      return result
    }
  }
}

const replaceNode = (node: TreeNode, newNode: TreeNode): TreeNode => {
  if (node.id === newNode.id) {
    return newNode
  }

  if (node.children === undefined) {
    return node
  }

  return {
    ...node,
    children: node.children.map((child) => replaceNode(child, newNode))
  }
}

const removeNode = (node: TreeNode, id: number): TreeNode => {
  if (node.id === id) {
    throw new Error('Cannot remove root node')
  }

  if (node.children === undefined) {
    return node
  }

  return {
    ...node,
    children: node.children
      .filter(child => child.id !== id) // remove node here
      .map(child => removeNode(child, id))
  }
}

const treeReducer = (state: TreeState, action: TreeAction): TreeState => {
  switch (action.type) {
    case 'ADD_NODE': {
      const parentNode = findNode(state.tree, action.payload.parentId)

      if (parentNode === undefined) {
        throw new Error('Parent node not found')
      }

      if (parentNode.children === undefined) {
        throw new Error('Children is undefined')
      }

      const lastTreeId = state.lastTreeId + 1
      const newNode: TreeNode = {
        ...action.payload.node,
        id: lastTreeId
      }

      const newParentNode = {
        ...parentNode,
        children: [
          ...parentNode.children,
          newNode
        ]
      }

      const newTree = replaceNode(state.tree, newParentNode)

      return {
        ...state,
        tree: newTree,
        lastTreeId
      }
    }
    case 'REMOVE_NODE': {
      const newTree = removeNode(state.tree, action.payload.node.id)

      return {
        ...state,
        tree: newTree
      }
    }
    case 'EDIT_NODE': {
      const newTree = replaceNode(state.tree, action.payload.node)

      return {
        ...state,
        tree: newTree
      }
    }
  }
}

export const TreeContext = createContext<{ state: TreeState, dispatch: Dispatch<TreeAction> }>({
  state: treeInitialState,
  dispatch: () => { /* empty */ }
})

interface TreeProviderProps {
  children?: React.ReactNode
}

export const TreeProvider: React.FC<TreeProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(treeReducer, treeInitialState)

  return (
    <TreeContext.Provider value={{ state, dispatch }}>
      {children}
    </TreeContext.Provider>
  )
}
