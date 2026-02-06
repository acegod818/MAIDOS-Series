// MAIDOS-Forge Studio - Dependency Graph Control
// Code-QC v2.2B Compliant

using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using Forge.Studio.ViewModels;

namespace Forge.Studio.Controls;

/// <summary>
/// Custom control for visualizing module dependencies as a directed graph
/// </summary>
public class DependencyGraphControl : Control
{
    public static readonly StyledProperty<ObservableCollection<ModuleViewModel>> ModulesProperty =
        AvaloniaProperty.Register<DependencyGraphControl, ObservableCollection<ModuleViewModel>>(nameof(Modules));

    public static readonly StyledProperty<ObservableCollection<DependencyLink>> DependenciesProperty =
        AvaloniaProperty.Register<DependencyGraphControl, ObservableCollection<DependencyLink>>(nameof(Dependencies));

    public ObservableCollection<ModuleViewModel> Modules
    {
        get => GetValue(ModulesProperty);
        set => SetValue(ModulesProperty, value);
    }

    public ObservableCollection<DependencyLink> Dependencies
    {
        get => GetValue(DependenciesProperty);
        set => SetValue(DependenciesProperty, value);
    }

    private readonly Dictionary<string, Point> _nodePositions = new();
    private readonly Dictionary<string, Color> _languageColors = new()
    {
        ["C#"] = Color.Parse("#512BD4"),
        ["Rust"] = Color.Parse("#DEA584"),
        ["C"] = Color.Parse("#555555"),
        ["C++"] = Color.Parse("#00599C"),
        ["Go"] = Color.Parse("#00ADD8"),
        ["Python"] = Color.Parse("#3776AB"),
        ["TypeScript"] = Color.Parse("#3178C6"),
        ["JavaScript"] = Color.Parse("#F7DF1E"),
        ["Zig"] = Color.Parse("#F7A41D"),
        ["Nim"] = Color.Parse("#FFE953"),
        ["Julia"] = Color.Parse("#9558B2"),
        ["Haskell"] = Color.Parse("#5E5086"),
        ["Ruby"] = Color.Parse("#CC342D"),
        ["Java"] = Color.Parse("#B07219"),
        ["Kotlin"] = Color.Parse("#A97BFF"),
        ["Swift"] = Color.Parse("#F05138"),
        ["Default"] = Color.Parse("#636E72")
    };

    public DependencyGraphControl()
    {
        // Enable pointer events
        IsHitTestVisible = true;
    }

    static DependencyGraphControl()
    {
        AffectsRender<DependencyGraphControl>(ModulesProperty, DependenciesProperty);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        
        if (change.Property == ModulesProperty || change.Property == DependenciesProperty)
        {
            CalculateLayout();
            InvalidateVisual();
        }
    }

    private void CalculateLayout()
    {
        _nodePositions.Clear();
        
        var modules = Modules;
        if (modules == null || modules.Count == 0)
            return;

        var width = Bounds.Width > 0 ? Bounds.Width : 800;
        var height = Bounds.Height > 0 ? Bounds.Height : 600;

        // Simple circular layout
        var count = modules.Count;
        var centerX = width / 2;
        var centerY = height / 2;
        var radius = Math.Min(width, height) * 0.35;

        for (int i = 0; i < count; i++)
        {
            var angle = 2 * Math.PI * i / count - Math.PI / 2;
            var x = centerX + radius * Math.Cos(angle);
            var y = centerY + radius * Math.Sin(angle);
            
            _nodePositions[modules[i].Name] = new Point(x, y);
            modules[i].X = x;
            modules[i].Y = y;
        }
    }

    public override void Render(DrawingContext context)
    {
        base.Render(context);

        // Draw background
        context.FillRectangle(new SolidColorBrush(Color.Parse("#2D3436")), new Rect(Bounds.Size));

        // Draw grid pattern
        DrawGrid(context);

        var modules = Modules;
        var dependencies = Dependencies;

        if (modules == null || modules.Count == 0)
        {
            // Draw placeholder text
            var text = new FormattedText(
                "Load a project to view dependency graph",
                System.Globalization.CultureInfo.CurrentCulture,
                FlowDirection.LeftToRight,
                new Typeface("Segoe UI"),
                14,
                new SolidColorBrush(Color.Parse("#636E72")));
            
            context.DrawText(text, new Point(
                (Bounds.Width - text.Width) / 2,
                (Bounds.Height - text.Height) / 2));
            return;
        }

        if (_nodePositions.Count == 0)
            CalculateLayout();

        // Draw dependency edges first (so they appear behind nodes)
        if (dependencies != null)
        {
            var edgePen = new Pen(new SolidColorBrush(Color.Parse("#636E72")), 2);
            
            foreach (var dep in dependencies)
            {
                if (_nodePositions.TryGetValue(dep.From, out var from) &&
                    _nodePositions.TryGetValue(dep.To, out var to))
                {
                    DrawArrow(context, from, to, edgePen);
                }
            }
        }

        // Draw nodes
        foreach (var module in modules)
        {
            if (_nodePositions.TryGetValue(module.Name, out var pos))
            {
                DrawNode(context, module, pos);
            }
        }
    }

    private void DrawGrid(DrawingContext context)
    {
        var gridPen = new Pen(new SolidColorBrush(Color.Parse("#1E272E")), 1);
        var gridSize = 40;

        for (double x = 0; x < Bounds.Width; x += gridSize)
        {
            context.DrawLine(gridPen, new Point(x, 0), new Point(x, Bounds.Height));
        }

        for (double y = 0; y < Bounds.Height; y += gridSize)
        {
            context.DrawLine(gridPen, new Point(0, y), new Point(Bounds.Width, y));
        }
    }

    private void DrawNode(DrawingContext context, ModuleViewModel module, Point center)
    {
        var nodeRadius = 35.0;
        var langKey = module.Language;
        
        if (!_languageColors.TryGetValue(langKey, out var color))
            color = _languageColors["Default"];

        // Draw node shadow
        var shadowBrush = new SolidColorBrush(Color.FromArgb(64, 0, 0, 0));
        context.DrawEllipse(shadowBrush, null,
            new Point(center.X + 3, center.Y + 3), nodeRadius, nodeRadius);

        // Draw node circle
        var nodeBrush = new SolidColorBrush(color);
        var borderPen = new Pen(new SolidColorBrush(Color.Parse("#DFE6E9")), 2);
        
        if (module.IsSelected)
        {
            borderPen = new Pen(new SolidColorBrush(Color.Parse("#FF6B35")), 3);
        }

        context.DrawEllipse(nodeBrush, borderPen, center, nodeRadius, nodeRadius);

        // Draw module name
        var nameText = new FormattedText(
            module.Name,
            System.Globalization.CultureInfo.CurrentCulture,
            FlowDirection.LeftToRight,
            new Typeface("Segoe UI", FontStyle.Normal, FontWeight.Bold),
            11,
            Brushes.White);

        context.DrawText(nameText, new Point(
            center.X - nameText.Width / 2,
            center.Y - nameText.Height / 2));

        // Draw language label below
        var langText = new FormattedText(
            module.Language,
            System.Globalization.CultureInfo.CurrentCulture,
            FlowDirection.LeftToRight,
            new Typeface("Segoe UI"),
            9,
            new SolidColorBrush(Color.Parse("#B2BEC3")));

        context.DrawText(langText, new Point(
            center.X - langText.Width / 2,
            center.Y + nodeRadius + 5));
    }

    private void DrawArrow(DrawingContext context, Point from, Point to, Pen pen)
    {
        // Calculate direction vector
        var dx = to.X - from.X;
        var dy = to.Y - from.Y;
        var length = Math.Sqrt(dx * dx + dy * dy);
        
        if (length == 0) return;

        // Normalize
        dx /= length;
        dy /= length;

        // Offset from node centers (to avoid overlapping with node circles)
        var nodeRadius = 35.0;
        var startOffset = nodeRadius + 5;
        var endOffset = nodeRadius + 15;

        var start = new Point(from.X + dx * startOffset, from.Y + dy * startOffset);
        var end = new Point(to.X - dx * endOffset, to.Y - dy * endOffset);

        // Draw line
        context.DrawLine(pen, start, end);

        // Draw arrowhead
        var arrowSize = 10.0;
        var angle = Math.Atan2(dy, dx);
        var arrowAngle = Math.PI / 6; // 30 degrees

        var arrow1 = new Point(
            end.X - arrowSize * Math.Cos(angle - arrowAngle),
            end.Y - arrowSize * Math.Sin(angle - arrowAngle));
        var arrow2 = new Point(
            end.X - arrowSize * Math.Cos(angle + arrowAngle),
            end.Y - arrowSize * Math.Sin(angle + arrowAngle));

        context.DrawLine(pen, end, arrow1);
        context.DrawLine(pen, end, arrow2);
    }

    protected override Size MeasureOverride(Size availableSize)
    {
        return availableSize;
    }

    protected override Size ArrangeOverride(Size finalSize)
    {
        CalculateLayout();
        return finalSize;
    }
}
