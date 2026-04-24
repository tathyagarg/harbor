// place files you want to import through the `$lib` alias in this folder.

export const GITHUB_URL = 'https://github.com/tathyagarg/harbor';

export const STEP_NAMES = [
  'Font',
  'HTTP',
  'HTML',
  'Link',
  'CSS',
  'Cascade',
  'Layout',
  'Rasterize',
  'Paint',
]

export const CORE_DATA: Record<string, {
  title: string,
  description: string,
  steps: {
    index: number,
    description: string
  }[]
}> = {
  'html': {
    'title': 'HTML',
    'description': 'HTML is the markup language that structures the content of a webpage. It defines elements such as headings, paragraphs, images, and links, allowing Harbor Browser to understand and render the content correctly.',
    'steps': [
      {
        index: 2,
        description: "Harbor Browser takes the raw HTML text and parses it into a Document Object Model (DOM) tree. This tree structure represents the hierarchical organization of the HTML elements, allowing Harbor to efficiently access and manipulate the content during rendering."
      },
      {
        index: 6,
        description: "Harbor Browser calculates the layout of the DOM tree, determining the position and size of each element based on CSS rules and the structure of the HTML. This step is crucial for rendering the webpage correctly, as it ensures that elements are displayed in the right place and with the appropriate dimensions."
      }
    ]
  }
}
