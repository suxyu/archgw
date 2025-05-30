.. _llm_router:

LLM Routing
==============================================================

LLM Router is an intelligent routing system that automatically selects the most appropriate large language model (LLM) for each user request based on the intent, domain, and complexity of the prompt. This enables optimal performance, cost efficiency, and response quality by matching requests with the most suitable model from your available LLM fleet.


Routing Workflow
-------------------------

#. **Prompt Analysis**

    When a user submits a prompt, the Router analyzes it to determine the domain (subject matter) or action (type of operation requested).

#. **Model Selection**

    Based on the analyzed intent and your configured routing preferences, the Router selects the most appropriate model from your available LLM fleet.

#. **Request Forwarding**

    Once the optimal model is identified, our gateway forwards the original prompt to the selected LLM endpoint. The routing decision is transparent and can be logged for monitoring and optimization purposes.

#. **Response Handling**

    After the selected model processes the request, the response is returned through the gateway. The gateway can optionally add routing metadata or performance metrics to help you understand and optimize your routing decisions.

Arch-Router
-------------------------
The `Arch-Router <https://huggingface.co/katanemo/Arch-Router-1.5B>`_ is a state-of-the-art **preference-based routing model** specifically designed for intelligent LLM selection. This model delivers production-ready performance with low latency and high accuracy.

To support effective routing, Arch-Router introduces two key concepts:

- **Domain** – the high-level thematic category or subject matter of a request (e.g., legal, healthcare, programming).

- **Action** – the specific type of operation the user wants performed (e.g., summarization, code generation, booking appointment, translation).

Both domain and action configs are associated with preferred models or model variants. At inference time, Arch-Router analyzes the incoming prompt to infer its domain and action using semantic similarity, task indicators, and contextual cues. It then applies the user-defined routing preferences to select the model best suited to handle the request.

In summary, Arch-Router demonstrates:

- **Structured Preference Routing**: Aligns prompt request with model strengths using explicit domain–action mappings.

- **Transparent and Controllable**: Makes routing decisions transparent and configurable, empowering users to customize system behavior.

- **Flexible and Adaptive**: Supports evolving user needs, model updates, and new domains/actions without retraining the router.

- **Production-Ready Performance**: Optimized for low-latency, high-throughput applications in multi-model environments.


Implementing LLM Routing
-----------------------------

To configure LLM routing in our gateway, you need to define a prompt target configuration that specifies the routing model and the LLM providers. This configuration will allow Arch Gateway to route incoming prompts to the appropriate model based on the defined routes.

Below is an example to show how to set up a prompt target for the Arch Router:

- **Step 1: Define the routing model in the `routing` section**. You can use the `archgw-v1-router-model` as the katanemo routing model or any other routing model you prefer.

- **Step 2: Define the listeners in the `listeners` section**. This is where you specify the address and port for incoming traffic, as well as the message format (e.g., OpenAI).

- **Step 3: Define the LLM providers in the `llm_providers` section**. This is where you specify the routing model, and any other models you want to use for specific tasks and their route usage descriptions (e.g., code generation, code understanding).

.. Note::
  Make sure you define a model for default usage, such as `gpt-4o`, which will be used when no specific route is matched for an user prompt.


.. code-block:: yaml
    :caption: Route Config Example


    routing:
    model: archgw-v1-router-model

    listeners:
    egress_traffic:
        address: 0.0.0.0
        port: 12000
        message_format: openai
        timeout: 30s

    llm_providers:
    - name: archgw-v1-router-model
        provider_interface: openai
        model: katanemo/Arch-Router-1.5B
        base_url: ...

    - name: gpt-4o-mini
        provider_interface: openai
        access_key: $OPENAI_API_KEY
        model: gpt-4o-mini
        default: true

    - name: code_generation
        provider_interface: openai
        access_key: $OPENAI_API_KEY
        model: gpt-4o
        usage: Generating new code snippets, functions, or boilerplate based on user prompts or requirements

    - name: code_understanding
        provider_interface: openai
        access_key: $OPENAI_API_KEY
        model: gpt-4.1
        usage: understand and explain existing code snippets, functions, or libraries


Example Use Cases
-------------------------
Here are common scenarios where Arch-Router excels:

- **Coding Tasks**: Distinguish between code generation requests ("write a Python function"), debugging needs ("fix this error"), and code optimization ("make this faster"), routing each to appropriately specialized models.

- **Content Processing Workflows**: Classify requests as summarization ("summarize this document"), translation ("translate to Spanish"), or analysis ("what are the key themes"), enabling targeted model selection.

- **Multi-Domain Applications**: Accurately identify whether requests fall into legal, healthcare, technical, or general domains, even when the subject matter isn't explicitly stated in the prompt.

- **Conversational Routing**: Track conversation context to identify when topics shift between domains or when the type of assistance needed changes mid-conversation.


Best practice
-------------------------
- **💡Consistent Naming:**  Route names should align with their descriptions.

  - ❌ Bad:
    ```
    {"name": "math", "description": "handle solving quadratic equations"}
    ```
  - ✅ Good:
    ```
    {"name": "quadratic_equation", "description": "solving quadratic equations"}
    ```

- **💡 Clear Usage Description:**  Make your route names and descriptions specific, unambiguous, and minimizing overlap between routes. The Router performs better when it can clearly distinguish between different types of requests.

  - ❌ Bad:
    ```
    {"name": "math", "description": "anything closely related to mathematics"}
    ```
  - ✅ Good:
    ```
    {"name": "math", "description": "solving, explaining math problems, concepts"}
    ```

- **💡Nouns Descriptor:** Preference-based routers perform better with noun-centric descriptors, as they offer more stable and semantically rich signals for matching.

- **💡Domain Inclusion:** for best user experience, you should always include domain route. This help the router fall back to domain when action is not

.. Unsupported Features
.. -------------------------

.. The following features are **not supported** by the Arch-Router model:

.. - **❌ Multi-Modality:**
..   The model is not trained to process raw image or audio inputs. While it can handle textual queries *about* these modalities (e.g., "generate an image of a cat"), it cannot interpret encoded multimedia data directly.

.. - **❌ Function Calling:**
..   This model is designed for **semantic preference matching**, not exact intent classification or tool execution. For structured function invocation, use models in the **Arch-Function-Calling** collection.

.. - **❌ System Prompt Dependency:**
..   Arch-Router routes based solely on the user’s conversation history. It does not use or rely on system prompts for routing decisions.
