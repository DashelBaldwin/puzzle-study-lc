<div align="center">
  <img src="https://i.postimg.cc/T2HcGcDG/icon.webp" width="100">
  <h1>Lichess Puzzle Study Generator</h1>
</div>

This is a tool that creates and uploads puzzle study sets via the Lichess API, intended for use by chess improvers employing [the woodpecker method](https://forwardchess.com/blog/what-is-the-woodpecker-method/).

Lichess lacks any sort of dedicated feature for organizing puzzle sets. If one wishes to remain within the Lichess ecosystem, the only solution is to use the study feature to store data. Manually creating puzzle study sets would be impractical and excessively time-consuming, however; fortunately, the available API endpoints make it feasible to automate this process.

While a proper UI is still being developed for this tool, a temporary UI has been created as a placeholder. This will allow you to access the tool's capabilities from the console. 

**Example:** https://lichess.org/study/xjipXf1Q/
(Follow the solution if you can't solve the first one!) => https://lichess.org/training/zOm2u

**Features:**
* Fetch puzzles by their IDs
* Fetch puzzles you recently missed in puzzle training
* Generate and upload a puzzle study from fetched puzzles

## Tech Used:

**Rust**

My motivation for creating this project was to familiarize myself with the Rust programming language; therefore, Rust is the exclusive language implemented. Given my limited familiarity with many of Rust’s features, I decided it would be much more beneficial to undertake a substantial project rather than solely follow tutorials. I intentionally selected a project that would require me to engage with Rust’s more complex aspects, including asynchronous tasks and comprehensive error handling.

At some point, I plan to pivot this project towards a web-based application, focusing on frontend development and ease of use by the general public.

## Lessons Learned:

Given this tool’s reliance on frequent interaction with the Lichess API, I dedicated substantial effort to understanding Rust’s approach to error handling. I was pleased to discover Rust’s strong emphasis on safe error management, particularly in the context of asynchronous tasks. While initially challenging, especially with concepts like futures and boxing errors, Rust’s structure and informative compiler warnings led me down the correct path regardless. Contrary to my expectations, managing errors from various asynchronous API calls was smoother than anticipated, involving minimal friction.

Gaining a deeper understanding of Rust's philosophy has greatly boosted my confidence in the language, and I plan to focus my learning efforts on using Rust for lower level development in the future.