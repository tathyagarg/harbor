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
  },
  'css': {
    'title': 'CSS',
    'description': 'CSS (Cascading Style Sheets) is a stylesheet language that describes the presentation of a webpage. It allows Harbor Browser to apply styles such as colors, fonts, and layouts to the HTML elements, enhancing the visual appearance of the webpage.',
    'steps': [
      {
        index: 4,
        description: "Harbor Browser parses the CSS text and constructs a CSS Object Model (CSSOM) tree. This tree represents the styles defined in the CSS, allowing Harbor to efficiently apply these styles to the corresponding HTML elements during rendering."
      },
      {
        index: 5,
        description: "Harbor Browser applies the cascading rules of CSS to determine which styles take precedence when multiple rules target the same element. This involves considering factors such as specificity, importance, and source order to ensure that the correct styles are applied to each element."
      }
    ]
  },
  'js': {
    'title': 'JavaScript',
    'description': 'JavaScript is a programming language that enables interactive and dynamic features on webpages. It allows Harbor Browser to execute scripts that can manipulate the DOM, handle user events, and communicate with servers, enhancing the functionality of the webpage. In Harbor Browser, JavaScript is still extremely unstable and is thus not included in the pipeline by default.',
    'steps': []
  },
  'http': {
    'title': 'HTTP',
    'description': 'HTTP (Hypertext Transfer Protocol) is the protocol used for communication between the browser and web servers. It allows Harbor Browser to send requests for resources such as HTML, CSS, JavaScript, and images, and receive responses that contain the requested data.',
    'steps': [
      {
        index: 1,
        description: "Harbor Browser initiates an HTTP request to the server hosting the webpage. This request includes information such as the URL, HTTP method (e.g., GET), and headers that provide additional context about the request."
      },
      {
        index: 3,
        description: "Harbor Browser identifies resources required for the webpage, such as CSS files specified in <link> tags in the HTML. It then initiates additional HTTP requests to fetch these resources, ensuring that all necessary data is available for rendering the webpage correctly."
      }
    ]
  },
  'font': {
    'title': 'Font',
    'description': 'Fonts are a crucial aspect of web design, as they determine the appearance of text on a webpage. Harbor Browser supports various font formats and allows web developers to specify custom fonts using CSS, enhancing the visual appeal and readability of the webpage.',
    'steps': [
      {
        index: 0,
        description: "Harbor Browser processes font requests specified in the CSS, such as those defined in @font-face rules. It initiates HTTP requests to fetch the font files from the server, ensuring that the necessary fonts are available for rendering the text on the webpage."
      },
      {
        index: 7,
        description: "Harbor Browser rasterizes the text on the webpage using the fetched font files. This involves converting the vector-based font data into pixel-based images that can be displayed on the screen, ensuring that the text is rendered accurately and with the intended appearance."
      }
    ]
  },
  'render': {
    'title': 'Render',
    'description': 'Rendering is the process of generating the visual representation of a webpage based on the HTML, CSS, and other resources. Harbor Browser takes the structured data from the DOM and CSSOM trees and applies the styles to create a visually appealing webpage that users can interact with.',
    'steps': [
      {
        index: 8,
        description: "Harbor Browser combines the layout information from the previous steps with the rasterized text and images to paint the final visual representation of the webpage on the screen. This step involves drawing each element in the correct order and applying any necessary effects, such as shadows or gradients, to create a polished and visually appealing webpage."
      }
    ]
  }
}
