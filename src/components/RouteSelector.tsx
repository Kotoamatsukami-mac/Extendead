import type { ResolvedRoute } from '../types/commands';

interface RouteSelectorProps {
  routes: ResolvedRoute[];
  selectedIndex: number | null;
  onSelect: (index: number) => void;
}

export function RouteSelector({ routes, selectedIndex, onSelect }: RouteSelectorProps) {
  if (routes.length <= 1) return null;

  return (
    <div className="route-selector" role="listbox" aria-label="Choose a route">
      <p className="route-selector__label">Open via</p>
      <ul className="route-selector__list">
        {routes.map((route, i) => (
          <li key={i}>
            <button
              className={`route-selector__item ${selectedIndex === i ? 'route-selector__item--selected' : ''}`}
              role="option"
              aria-selected={selectedIndex === i}
              onClick={() => onSelect(i)}
            >
              <span className="route-selector__item-label">{route.label}</span>
              <span className="route-selector__item-desc">{route.description}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
