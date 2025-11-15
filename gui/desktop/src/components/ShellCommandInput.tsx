/**
 * Shell command input component
 */

import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { shellApi, ShellCommandRequest } from '@/lib/api';
import { Send, Loader2 } from 'lucide-react';

interface ShellCommandInputProps {
  disabled?: boolean;
}

export function ShellCommandInput({ disabled }: ShellCommandInputProps) {
  const [command, setCommand] = useState('');
  const [commandHistory, setCommandHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [lastResponse, setLastResponse] = useState<string[]>([]);

  const executeCommand = useMutation({
    mutationFn: (request: ShellCommandRequest) => shellApi.exec(request),
    onSuccess: (response) => {
      setLastResponse(response.output);
      if (command.trim()) {
        setCommandHistory((prev) => [...prev, command]);
      }
      setCommand('');
      setHistoryIndex(-1);
    },
    onError: (error: any) => {
      setLastResponse([
        `Error: ${error.response?.data?.error || error.message}`,
      ]);
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!command.trim() || disabled) return;

    executeCommand.mutate({
      command: command.trim(),
      timeout_ms: 30000,
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    // Arrow up/down for command history
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (commandHistory.length > 0) {
        const newIndex =
          historyIndex === -1
            ? commandHistory.length - 1
            : Math.max(0, historyIndex - 1);
        setHistoryIndex(newIndex);
        setCommand(commandHistory[newIndex]);
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIndex >= 0) {
        const newIndex = historyIndex + 1;
        if (newIndex >= commandHistory.length) {
          setHistoryIndex(-1);
          setCommand('');
        } else {
          setHistoryIndex(newIndex);
          setCommand(commandHistory[newIndex]);
        }
      }
    }
  };

  return (
    <div className="bg-card rounded-lg border p-4">
      <h3 className="text-lg font-semibold mb-4">Shell Command</h3>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="flex gap-2">
          <input
            type="text"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={disabled || executeCommand.isPending}
            placeholder="Enter shell command (e.g., help, info, autoctl status)"
            className="flex-1 px-3 py-2 text-sm border rounded-md bg-background disabled:opacity-50"
          />
          <button
            type="submit"
            disabled={
              disabled || !command.trim() || executeCommand.isPending
            }
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 flex items-center gap-2"
          >
            {executeCommand.isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Running...
              </>
            ) : (
              <>
                <Send className="h-4 w-4" />
                Execute
              </>
            )}
          </button>
        </div>

        {/* Command response */}
        {(lastResponse.length > 0 || executeCommand.error) && (
          <div className="bg-muted rounded-md p-3 max-h-60 overflow-y-auto">
            <div className="font-mono text-sm space-y-1">
              {lastResponse.map((line, i) => (
                <div key={i} className="text-foreground">
                  {line}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Command history hint */}
        {commandHistory.length > 0 && (
          <p className="text-xs text-muted-foreground">
            Use ↑/↓ arrows to navigate command history ({commandHistory.length}{' '}
            commands)
          </p>
        )}
      </form>
    </div>
  );
}
