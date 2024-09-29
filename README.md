# Pixlie AI
graph + ai in your products; reduce costs and get correct answers from your data

## What is Pixlie AI?
Pixlie AI is a framework to create applications that use a graph + S/LLM (Small/Large Language Model) approach
to semantic search, knowledge base, question/answering, and more.

These are some high level bullet points that highlight our approach to the problems and kind of solutions we want to bring about:
- We are building a system that would "**understand your data**" very well
- Our approach, with Pixlie AI, is to create a **knowledge graph on your data**
- Pixlie AI is **source available**, *converts to open source* 2 years after each release - **no vendor lock in**
- We are **not** trying to be a **generic AI solution** which can deliver answers to every question out there
- Pixlie AI aims to give **accurate answers** where **semantics** matter (meaning and relation of things)
- **Fast, reliable and low cost** - use Rust, deterministic cache, small language models, NLP tools, efficient storage, memory and compute management
- Large (and Small) Language Models help **extract entities** (people, places, events, time, tasks,...) from data
- Pixlie AI graph can manage **millions of entities**, 100s of millions of relations to hold semantics of data
- **Input question** translated to graph operations (using S/L LMs) and answered
- **Internal router** to guess model performance for given task - small models can further lower time and financial costs
- Output **answer has entity labels** (person, place, time,...) which can help in better UX and further workflow

Pixlie AI is being build as batteries-included and ready to use out of the box for majority of use-cases. Like Redis, or PostgreSQL.

## What does this solve?
Ever had a conversation with an LLM and saw that the answers are not always correct?

This is exactly what we want to solve. The graph is a rich set of connections between your data and all the
real world attributes they have. LLMs are great at finding details in data but are not so good at keeping
track of everything. The graph stores and retrieves all the data and metadata (from LLMs) deterministically.

This is an alternative to using vector databases for RAG (Retreival Augmented Generation).

## How does it work?
Pixlie AI is a framework that you can use in your products, share data to it and tweak the graph using the API.
Pixlie AI takes care of:
- Data storage with RocksDB (consider Pixlie AI data storage to be ephimeral)
- APIs to send your data to it (similar to search engines)
- Graph creation and management
- Integration with Large Language Models (your API keys)
- Entity and relationship extraction from your data
- Enrich the graph with extracted entities and relationships
- Query the graph using an API or natural language
- API to browse the graph and all related data

## License
- Pixlie AI is licensed under Business Source License 1.1 (BSL-1.1)
- See the [LICENSE](LICENSE) file for details
- Every release of Pixlie AI converts to Apache License 2.0 after two years

## Can I use Pixlie AI to power my product?
Yes! you can use Pixlie AI to power your SaaS or desktop applications.
There is no restriction except unless you are competing with our paid service(s).
Please see the [LICENSE](LICENSE) file for details.
