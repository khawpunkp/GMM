// src/components/EntityCard.jsx
import React from "react";
import { Link } from "react-router-dom";

// Helper function to parse details JSON
const parseDetails = (detailsJson) => {
  try {
    if (!detailsJson) return {};
    return JSON.parse(detailsJson);
  } catch (e) {
    console.error("Failed to parse entity details JSON:", e);
    return {}; // Return empty object on error
  }
};

const attributeIconsSrc = {
  Physical: "/images/filters/zzz/phisical.webp",
  Fire: "/images/filters/zzz/fire.webp",
  Ice: "/images/filters/zzz/ice.webp",
  Frost: "/images/filters/zzz/frost.webp",
  Electric: "/images/filters/zzz/electric.webp",
  Ether: "/images/filters/zzz/ether.webp",
  AuricInk: "/images/filters/zzz/auric-ink.webp",
  // Add more ZZZ attributes as needed
};

const specialityIconsSrc = {
  Attack: "/images/filters/zzz/attack.webp",
  Stun: "/images/filters/zzz/stun.webp",
  Anomaly: "/images/filters/zzz/anomaly.webp",
  Support: "/images/filters/zzz/support.webp",
  Defense: "/images/filters/zzz/defense.webp",
  Rupture: "/images/filters/zzz/rupture.webp",
};

const DEFAULT_PLACEHOLDER_IMAGE = "/images/unknown.jpg";

function EntityCard({ entity }) {
  // Destructure props including counts
  const {
    slug,
    name,
    details: detailsJson,
    base_image,
    total_mods,
    enabled_mods,
  } = entity;

  const details = parseDetails(detailsJson);

  const getRarityColor = () => {
    if (!details.rank) return "#888"; // Default color for unknown rarity
    const val = details.rank.toLowerCase();
    if (val === "5 star" || val === "s") return "#ffcc00"; // Gold
    if (val === "4 star" || val === "a") return "#a259ec"; // Purple
    return "#888"; // Gray for fallback
  };

  // ZZZ-specific properties
  const attribute = details?.attribute;
  const attributeIconClass = attribute ? attributeIconsSrc[attribute] : null;
  const speciality = details?.speciality;
  const specialtyIconClass = speciality ? specialityIconsSrc[speciality] : null;

  const imageUrl = base_image
    ? `/images/entities/${slug}_base.jpg`
    : DEFAULT_PLACEHOLDER_IMAGE;

  return (
    <Link
      to={`/entity/${slug}`}
      className={`character-card zzz-card`}
      title={`View mods for ${name}`}
    >
      {/* Container for Badges (CSS will handle layout) */}
      <div className="card-badges-container">
        {/* Total Mod Count Badge */}
        {total_mods > 0 && (
          <div
            className="card-badge total-badge"
            title={`${total_mods} total mods`}
          >
            {total_mods}{" "}
            <i
              className="fas fa-box fa-fw"
              style={{ marginLeft: "3px", opacity: 0.8 }}
            ></i>
          </div>
        )}
        {/* Enabled Mod Count Badge */}
        {enabled_mods > 0 && (
          <div
            className="card-badge enabled-badge"
            title={`${enabled_mods} mods enabled`}
          >
            {enabled_mods}{" "}
            <i
              className="fas fa-check-circle fa-fw"
              style={{ marginLeft: "3px" }}
            ></i>
          </div>
        )}
      </div>

      {/* Card Image */}
      <div
        className="card-image"
        style={{
          backgroundImage: `url('${imageUrl}')`,
          borderBottomLeftRadius: "12px",
          borderBottomRightRadius: "12px",
        }}
        // onError could potentially be added here if using <img> instead of background
      >
        {specialtyIconClass && (
          <div
            style={{
              width: "36px",
              height: "36px",
              borderRadius: "999px",
              padding: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: "#1f1e36",
              marginRight: "4px",
            }}
          >
            <img src={specialtyIconClass} />
          </div>
        )}
        {attributeIconClass && (
          <div
            style={{
              width: "36px",
              height: "36px",
              borderRadius: "999px",
              padding: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: "#1f1e36",
            }}
          >
            <img src={attributeIconClass} />
          </div>
        )}
      </div>

      <div
        style={{
          height: "40px",
          marginTop: "-16px",
          backgroundColor: getRarityColor(),
          borderBottomLeftRadius: "12px",
          borderBottomRightRadius: "12px",
        }}
      />

      {/* Card Content */}
      <div className="card-content">
        <div className="card-name">{name}</div>
      </div>
    </Link>
  );
}

export default EntityCard;
