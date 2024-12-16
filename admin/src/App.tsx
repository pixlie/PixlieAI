import Sidebar from "./layout/Sidebar";

function App(props: any) {
  return (
    <div class="relative isolate flex min-h-svh w-full bg-white">
      <Sidebar />

      <div class="pl-72 flex-1">{props.children}</div>
    </div>
  );
}

export default App;
