# Test Mermaid Diagrams

## Test 1: Simple Flowchart

```mermaid
graph TD
    A[Start] --> B{Is it working?}
    B -->|Yes| C[Great!]
    B -->|No| D[Debug]
    D --> A
    C --> E[End]
```

## Test 2: Sequence Diagram

```mermaid
sequenceDiagram
    Alice->>John: Hello John, how are you?
    John-->>Alice: Great!
    Alice-)John: See you later!
```

## Test 3: Class Diagram

```mermaid
classDiagram
    Animal <|-- Duck
    Animal <|-- Fish
    Animal : +int age
    Animal : +String gender
    Animal: +isMammal()
    class Duck{
        +String beakColor
        +swim()
        +quack()
    }
    class Fish{
        -int sizeInFeet
        -canEat()
    }
```

## Test 4: State Diagram

```mermaid
stateDiagram-v2
    [*] --> Still
    Still --> [*]
    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
```

## How to Test

1. Open this file in Zed
2. Copy one of the diagram blocks (the text between ```mermaid and ```)
3. In Zed's Assistant panel, type:
   `/mermaid-preview <paste diagram here>`
4. The extension will render the diagram and give you a file path
5. Open the file path to see the SVG preview

## Expected Output

You should see a message like:
```
✅ Diagram rendered successfully

Preview: /Users/wfukatsu/.cache/zed/mermaid/abc123...svg

Open with your system viewer.
```

Then you can open that file with:
```bash
open /Users/wfukatsu/.cache/zed/mermaid/abc123...svg
```
