import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

// Minimal stand-ins for the BIGSU components so the test exercises LoginScreen
// behavior, not the design system.
vi.mock("@gio/bigsu-ui", () => ({
  Card: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
  CardDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  FormField: ({
    label,
    errorText,
    children,
  }: {
    label: string;
    errorText?: string;
    children: React.ReactNode;
  }) => (
    <div>
      <label>
        {label}
        {children}
      </label>
      {errorText ? <span role="alert">{errorText}</span> : null}
    </div>
  ),
  Input: (props: Record<string, unknown>) => <input {...props} />,
  Checkbox: ({
    label,
    checked,
    onCheckedChange,
  }: {
    label: string;
    checked: boolean;
    onCheckedChange: (c: boolean) => void;
  }) => (
    <label>
      {label}
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onCheckedChange(e.target.checked)}
      />
    </label>
  ),
  Button: ({
    children,
    onClick,
    loading,
    disabled,
    type,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    loading?: boolean;
    disabled?: boolean;
    type?: "button" | "submit";
  }) => (
    <button type={type} onClick={onClick} disabled={loading || disabled}>
      {children}
    </button>
  ),
}));

import LoginScreen from "../LoginScreen";

describe("LoginScreen", () => {
  beforeEach(() => vi.clearAllMocks());

  function fill(username: string, password: string) {
    fireEvent.change(screen.getByLabelText("Username"), { target: { value: username } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: password } });
  }

  it("prefills the saved username", () => {
    render(<LoginScreen defaultUsername="rendy" onLogin={vi.fn()} />);
    expect(screen.getByLabelText("Username")).toHaveValue("rendy");
  });

  it("shows the initial error from a failed auto-login", () => {
    render(<LoginScreen initialError="Saved password is no longer valid." onLogin={vi.fn()} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Saved password is no longer valid.");
  });

  it("disables submit until both fields are filled", () => {
    render(<LoginScreen onLogin={vi.fn()} />);
    const button = screen.getByRole("button", { name: "Sign in" });
    expect(button).toBeDisabled();
    fill("rendy", "secret");
    expect(button).not.toBeDisabled();
  });

  it("calls onLogin with trimmed username, password, and remember flag", async () => {
    const onLogin = vi.fn().mockResolvedValue(undefined);
    render(<LoginScreen onLogin={onLogin} />);
    fill("  rendy  ", "secret");
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    await waitFor(() => {
      expect(onLogin).toHaveBeenCalledWith("rendy", "secret", true);
    });
  });

  it("surfaces the backend error string and clears the password", async () => {
    const onLogin = vi.fn().mockRejectedValue("Invalid username or password.");
    render(<LoginScreen onLogin={onLogin} />);
    fill("rendy", "wrong");
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveTextContent("Invalid username or password.");
    });
    expect(screen.getByLabelText("Password")).toHaveValue("");
  });

  it("does not submit when remember is unchecked -> passes false", async () => {
    const onLogin = vi.fn().mockResolvedValue(undefined);
    render(<LoginScreen onLogin={onLogin} />);
    fill("rendy", "secret");
    fireEvent.click(screen.getByLabelText("Remember me on this device"));
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    await waitFor(() => {
      expect(onLogin).toHaveBeenCalledWith("rendy", "secret", false);
    });
  });
});
