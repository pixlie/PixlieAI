import { Component } from "solid-js";
import { APIProvider } from "../../api_types/APIProvider";
import ApiKey from "./ApiKey";

const ApiKeys: Component = () => {
  return (
    <div class="space-y-4">
      <div class="">
        <h3 class="text-lg font-medium text-slate-800 flex items-center gap-2">
          {/* <svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
          </svg> */}
          API Keys
        </h3>
        {/* <p class="text-slate-600 text-sm">Configure your API keys to enable AI features and web search capabilities.</p> */}
      </div>
      
      <div class="space-y-4">
        <ApiKey 
          provider={"Anthropic" as APIProvider}
          title="Anthropic"
          instructions={{
            accountUrl: "https://console.anthropic.com/login",
            accountText: "here",
            keyUrl: "https://console.anthropic.com/settings/keys",
            keyText: "here",
            placeholder: "sk-ant-...",
            validate: (key: string) => ({
              isValid: key.length >= 64 && key.startsWith("sk-ant-"),
              errorMessage: "Please enter a valid Anthropic API key (starts with sk-ant-)"
            })
          }}
        />
        
        <ApiKey 
          provider={"BraveSearch" as APIProvider}
          title="Brave Search"
          instructions={{
            accountUrl: "https://brave.com/search/api/",
            accountText: "here",
            keyUrl: "https://api-dashboard.search.brave.com/app/keys",
            keyText: "here",
            placeholder: "BSA...",
            validate: (key: string) => ({
              isValid: key.length >= 30 && key.startsWith("BSA"),
              errorMessage: "Please enter a valid Brave Search API key (starts with BSA)"
            })
          }}
        />
        
        <ApiKey 
          provider={"SendGrid" as APIProvider}
          title="SendGrid"
          instructions={{
            accountUrl: "https://signup.sendgrid.com/",
            accountText: "here",
            keyUrl: "https://app.sendgrid.com/settings/api_keys",
            keyText: "here",
            placeholder: "SG...",
            validate: (key: string) => ({
              isValid: key.startsWith("SG."),
              errorMessage: "Please enter a valid SendGrid API key (starts with SG.)"
            })
          }}
        />
      </div>
    </div>
  );
};

export default ApiKeys;
